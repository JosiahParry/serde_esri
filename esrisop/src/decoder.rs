use crate::{
    error::Error,
    metadata::{ColumnPath, GeodisplayJson, GeodisplayMetadata},
    plan::{rows, Plan},
    query::Query,
    xz::XzRanges,
};
use arrow_array::{
    builder::BinaryBuilder,
    cast::AsArray,
    types::{Float64Type, Int64Type},
    Array, BooleanArray, Float64Array, RecordBatch,
};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use arrow_select::filter::filter_record_batch;
use bytes::Bytes;
use esripbf::{convert::geometry::Quantization, esri_p_buffer::feature_collection_p_buffer as pbf};
use parquet::{
    arrow::{
        arrow_reader::{ArrowPredicateFn, ArrowReaderMetadata, RowFilter},
        push_decoder::{ParquetPushDecoder, ParquetPushDecoderBuilder},
        ProjectionMask,
    },
    DecodeResult,
};
use prost::Message;
use serde_esri::enginex::{
    description::VertexDescription,
    geometry::{Envelope2D, Geometry, Point},
    shape::writer::EsriShapeBuffer,
    vertex::Vertex,
};
use std::{
    collections::{BTreeSet, HashMap},
    ops::Range,
    sync::Arc,
};

enum GeometrySource {
    Level {
        column: ColumnPath,
        quantization: Quantization,
        geometry_type: pbf::GeometryType,
        extents: Vec<Envelope2D>,
    },
    Points {
        x: ColumnPath,
        y: ColumnPath,
        z: Option<ColumnPath>,
        m: Option<ColumnPath>,
    },
}

pub struct Decoder {
    decoder: ParquetPushDecoder,
    schema: SchemaRef,
    columns: Vec<String>,
    geometry: Option<GeometrySource>,
}

impl Decoder {
    pub fn try_new(metadata: ArrowReaderMetadata, query: &Query) -> Result<Self, Error> {
        let raw = GeodisplayJson::try_from(metadata.metadata().file_metadata())?;
        let geodisplay = GeodisplayMetadata::try_from(&raw)?;
        let descriptor = metadata.parquet_schema();
        let arrow_schema = metadata.schema();

        let (display, geometry, index_columns) = match &geodisplay {
            GeodisplayMetadata::Xz(xz) => {
                let geometry = query
                    .resolution
                    .map(|resolution| xz.level(resolution))
                    .transpose()?
                    .map(|level| GeometrySource::Level {
                        column: level.column.clone(),
                        quantization: Quantization::new(
                            &pbf::Transform::from(&level.transform),
                            VertexDescription::from(&xz.display),
                        ),
                        geometry_type: xz.geometry_type.into(),
                        extents: query.extents.clone(),
                    });
                let columns = std::iter::once(&xz.code)
                    .chain(xz.levels.iter().map(|level| &level.column))
                    .collect::<Vec<_>>();
                (&xz.display, geometry, columns)
            }
            GeodisplayMetadata::Z(z) => {
                let geometry = GeometrySource::Points {
                    x: z.x_column.clone(),
                    y: z.y_column.clone(),
                    z: z.z_column.clone().filter(|_| z.display.has_z),
                    m: z.m_column.clone().filter(|_| z.display.has_m),
                };
                let columns = [
                    Some(&z.code),
                    Some(&z.x_column),
                    Some(&z.y_column),
                    z.z_column.as_ref(),
                    z.m_column.as_ref(),
                ]
                .into_iter()
                .flatten()
                .filter(|path| matches!(path, ColumnPath::Child(..)) || *path == &z.code)
                .collect::<Vec<_>>();
                (&z.display, Some(geometry), columns)
            }
        };

        let hidden = index_columns
            .into_iter()
            .map(|path| match path {
                ColumnPath::Field(name) | ColumnPath::Child(name, _) => name.as_str(),
            })
            .collect::<BTreeSet<_>>();
        let columns = match &query.columns {
            Some(columns) => columns.clone(),
            None => arrow_schema
                .fields()
                .iter()
                .map(|field| field.name().clone())
                .filter(|name| !hidden.contains(name.as_str()))
                .collect(),
        };

        let mut fields = Vec::with_capacity(columns.len() + 1);
        let mut leaves = BTreeSet::new();
        for name in &columns {
            let (root, field) = arrow_schema
                .column_with_name(name)
                .ok_or_else(|| Error::MissingColumn(name.clone()))?;
            fields.push(Arc::new(field.clone()));
            leaves.extend(
                (0..descriptor.num_columns())
                    .filter(|&leaf| descriptor.get_column_root_idx(leaf) == root),
            );
        }
        match &geometry {
            Some(GeometrySource::Level { column, .. }) => {
                leaves.insert(column.leaf(descriptor)?);
            }
            Some(GeometrySource::Points { x, y, z, m }) => {
                for path in [Some(x), Some(y), z.as_ref(), m.as_ref()]
                    .into_iter()
                    .flatten()
                {
                    leaves.insert(path.leaf(descriptor)?);
                }
            }
            None => {}
        }
        if geometry.is_some() {
            let metadata = HashMap::from([
                ("esri.encoding".to_string(), "EsriShape".to_string()),
                (
                    "esri.sr_wkt".to_string(),
                    display.wkt.clone().unwrap_or_default(),
                ),
                ("geodisplay".to_string(), raw.json.clone()),
            ]);
            fields.push(Arc::new(
                Field::new("geometry", DataType::Binary, true).with_metadata(metadata),
            ));
        }

        let parquet_metadata = metadata.metadata().clone();
        let mut builder = ParquetPushDecoderBuilder::new_with_metadata(metadata.clone())
            .with_projection(ProjectionMask::leaves(descriptor, leaves));
        if !query.extents.is_empty() {
            let (plan, filter) = match &geodisplay {
                GeodisplayMetadata::Xz(xz) => {
                    let ranges = XzRanges::new(
                        &Envelope2D::from(&xz.display.full_extent),
                        &query.extents,
                        xz.max_level,
                    );
                    let leaf = xz.code.leaf(descriptor)?;
                    let plan = Plan::new(&parquet_metadata, |row_group, index| {
                        rows(&parquet_metadata, row_group, index, leaf, |[min, max]| {
                            ranges.overlaps(min, max)
                        })
                    });
                    let path = xz.code.clone();
                    let predicate = ArrowPredicateFn::new(
                        ProjectionMask::leaves(descriptor, [leaf]),
                        move |batch: RecordBatch| {
                            let codes = path
                                .column(&batch)?
                                .as_primitive_opt::<Int64Type>()
                                .ok_or_else(|| Error::ColumnType {
                                    column: path.to_string(),
                                    expected: "64-bit integer",
                                })?;
                            Ok(codes
                                .iter()
                                .map(|code| Some(code.is_some_and(|code| ranges.contains(code))))
                                .collect::<BooleanArray>())
                        },
                    );
                    (plan, RowFilter::new(vec![Box::new(predicate)]))
                }
                GeodisplayMetadata::Z(z) => {
                    let (x, y) = (z.x_column.leaf(descriptor)?, z.y_column.leaf(descriptor)?);
                    let plan = Plan::new(&parquet_metadata, |row_group, index| {
                        query
                            .extents
                            .iter()
                            .map(|e| {
                                rows::<f64>(&parquet_metadata, row_group, index, x, |[lo, hi]| {
                                    lo <= e.xmax && hi >= e.xmin
                                })
                                .intersection(&rows::<f64>(
                                    &parquet_metadata,
                                    row_group,
                                    index,
                                    y,
                                    |[lo, hi]| lo <= e.ymax && hi >= e.ymin,
                                ))
                            })
                            .reduce(|a, b| a.union(&b))
                            .unwrap_or_default()
                    });
                    let (x_path, y_path) = (z.x_column.clone(), z.y_column.clone());
                    let extents = query.extents.clone();
                    let predicate = ArrowPredicateFn::new(
                        ProjectionMask::leaves(descriptor, [x, y]),
                        move |batch: RecordBatch| {
                            let xs = doubles(&x_path, &batch)?;
                            let ys = doubles(&y_path, &batch)?;
                            Ok(xs
                                .iter()
                                .zip(ys.iter())
                                .map(|point| match point {
                                    (Some(x), Some(y)) => Some(extents.iter().any(|e| {
                                        x >= e.xmin && x <= e.xmax && y >= e.ymin && y <= e.ymax
                                    })),
                                    _ => Some(false),
                                })
                                .collect::<BooleanArray>())
                        },
                    );
                    (plan, RowFilter::new(vec![Box::new(predicate)]))
                }
            };
            builder = builder
                .with_row_groups(plan.row_groups)
                .with_row_selection(plan.selection)
                .with_row_filter(filter);
        }

        Ok(Decoder {
            decoder: builder.build()?,
            schema: Arc::new(Schema::new(fields)),
            columns,
            geometry,
        })
    }

    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    pub fn try_decode(&mut self) -> Result<DecodeResult<RecordBatch>, Error> {
        loop {
            let batch = match self.decoder.try_decode()? {
                DecodeResult::Data(batch) => batch,
                DecodeResult::NeedsData(ranges) => return Ok(DecodeResult::NeedsData(ranges)),
                DecodeResult::Finished => return Ok(DecodeResult::Finished),
            };
            let mut columns = self
                .columns
                .iter()
                .map(|name| {
                    batch
                        .column_by_name(name)
                        .cloned()
                        .ok_or_else(|| Error::MissingColumn(name.clone()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut keep = None;
            let rows = batch.num_rows();
            match &self.geometry {
                Some(GeometrySource::Level {
                    column,
                    quantization,
                    geometry_type,
                    extents,
                }) => {
                    let values = column.column(&batch)?;
                    let bytes = match (values.as_binary_opt::<i32>(), values.as_binary_opt::<i64>())
                    {
                        (Some(values), _) => values.iter().collect::<Vec<_>>(),
                        (None, Some(values)) => values.iter().collect(),
                        (None, None) => values
                            .as_binary_view_opt()
                            .ok_or_else(|| Error::ColumnType {
                                column: column.to_string(),
                                expected: "binary",
                            })?
                            .iter()
                            .collect(),
                    };
                    let mut shapes = BinaryBuilder::with_capacity(rows, 0);
                    let mut hits = Vec::with_capacity(rows);
                    for bytes in bytes {
                        let message = bytes.map(pbf::Geometry::decode).transpose()?;
                        let geometry = message
                            .filter(|message| !message.coords.is_empty())
                            .map(|message| quantization.engine(*geometry_type, &message))
                            .transpose()?;
                        let Some(geometry) = geometry else {
                            shapes.append_null();
                            hits.push(false);
                            continue;
                        };
                        shapes.append_value(EsriShapeBuffer::try_from(&geometry)?.0);
                        hits.push(geometry.envelope().xy.is_some_and(|b| {
                            extents.iter().any(|e| {
                                b.xmin <= e.xmax
                                    && b.xmax >= e.xmin
                                    && b.ymin <= e.ymax
                                    && b.ymax >= e.ymin
                            })
                        }));
                    }
                    if !extents.is_empty() {
                        keep = Some(BooleanArray::from(hits));
                    }
                    columns.push(Arc::new(shapes.finish()));
                }
                Some(GeometrySource::Points { x, y, z, m }) => {
                    let (xs, ys) = (doubles(x, &batch)?, doubles(y, &batch)?);
                    let zs = z.as_ref().map(|z| doubles(z, &batch)).transpose()?;
                    let ms = m.as_ref().map(|m| doubles(m, &batch)).transpose()?;
                    let mut shapes = BinaryBuilder::with_capacity(rows, 0);
                    for row in 0..rows {
                        let value = |values: &Float64Array| {
                            values
                                .is_valid(row)
                                .then(|| values.value(row))
                                .filter(|v| !v.is_nan())
                        };
                        let (Some(x), Some(y)) = (value(xs), value(ys)) else {
                            shapes.append_null();
                            continue;
                        };
                        let point = Geometry::Point(Point(Some(Vertex {
                            x,
                            y,
                            z: zs.map(|zs| value(zs).unwrap_or(f64::NAN)),
                            m: ms.map(|ms| value(ms).unwrap_or(f64::NAN)),
                            id: None,
                        })));
                        shapes.append_value(EsriShapeBuffer::try_from(&point)?.0);
                    }
                    columns.push(Arc::new(shapes.finish()));
                }
                None => {}
            }
            let batch = RecordBatch::try_new(self.schema.clone(), columns)?;
            let batch = match keep {
                Some(keep) => filter_record_batch(&batch, &keep)?,
                None => batch,
            };
            if batch.num_rows() > 0 {
                return Ok(DecodeResult::Data(batch));
            }
        }
    }

    pub fn push_ranges(&mut self, ranges: Vec<Range<u64>>, data: Vec<Bytes>) -> Result<(), Error> {
        Ok(self.decoder.push_ranges(ranges, data)?)
    }
}

fn doubles<'a>(path: &ColumnPath, batch: &'a RecordBatch) -> Result<&'a Float64Array, Error> {
    path.column(batch)?
        .as_primitive_opt::<Float64Type>()
        .ok_or_else(|| Error::ColumnType {
            column: path.to_string(),
            expected: "double",
        })
}

#[cfg(test)]
mod tests;
