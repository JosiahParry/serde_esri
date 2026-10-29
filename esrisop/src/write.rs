use crate::{
    error::Error,
    metadata::{
        ColumnLookup, DisplayMetadata, Encoding, GeodisplayMetadata, MultiscaleLevel,
        QuantizationTransform, Writer, XzMetadata, ZMetadata,
    },
    xz::MaxLevel,
};
use arrow_array::{
    builder::BinaryBuilder, ArrayRef, Float64Array, Int64Array, RecordBatch, StructArray,
    UInt64Array,
};
use arrow_schema::{DataType, Field, Fields, Schema};
use arrow_select::take::take;
use esripbf::{
    convert::{geometry::Quantization, ToPbfError},
    esri_p_buffer::feature_collection_p_buffer as pbf,
};
use parquet::{
    file::{
        metadata::KeyValue,
        properties::{EnabledStatistics, WriterProperties, WriterPropertiesBuilder},
    },
    schema::types::ColumnPath,
};
use prost::Message;
use serde_esri::{
    enginex::{
        description::VertexDescription,
        flags::GeometryType,
        geometry::{Envelope, Envelope2D, Geometry, Point, Polygon, Polyline},
        shape::EsriShape,
        vertex::Vertex,
    },
    geometry::EsriEnvelope,
};
use std::sync::Arc;

const GROUP: &str = "geodisplay";
const WEB_MERCATOR_WIDTH: f64 = 20_037_508.342_789_244 * 2.0;

pub struct DisplayOptimization {
    pub geometry_column: String,
    pub wkid: i32,
    pub full_extent: Option<Envelope2D>,
}

pub struct Optimized {
    pub batch: RecordBatch,
    pub geodisplay: GeodisplayMetadata,
}

impl DisplayOptimization {
    pub fn apply(&self, batch: &RecordBatch) -> Result<Optimized, Error> {
        if batch.schema().column_with_name(GROUP).is_some() {
            return Err(Error::ColumnExists(GROUP.to_string()));
        }
        let geometries = ColumnPath::new(vec![self.geometry_column.clone()])
            .binaries(batch)?
            .into_iter()
            .map(|bytes| {
                bytes
                    .map(|bytes| Option::<Geometry>::try_from(EsriShape(bytes)))
                    .transpose()
                    .map(Option::flatten)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let geometry_type = geometries
            .iter()
            .flatten()
            .map(Geometry::geometry_type)
            .try_fold(None, |found, geometry_type| match found {
                Some(first) if first != geometry_type => {
                    Err(Error::MixedGeometryTypes(first, geometry_type))
                }
                _ => Ok(Some(geometry_type)),
            })?
            .ok_or(Error::NoGeometries)?;
        let envelopes = geometries
            .iter()
            .map(|geometry| geometry.as_ref().map(Geometry::envelope))
            .collect::<Vec<_>>();
        let has_z = envelopes
            .iter()
            .flatten()
            .any(|envelope| envelope.z.is_some());
        let has_m = envelopes
            .iter()
            .flatten()
            .any(|envelope| envelope.m.is_some());
        let full_extent = match self.full_extent {
            Some(extent) => extent,
            None => envelopes
                .iter()
                .flatten()
                .filter_map(|envelope| envelope.xy)
                .flat_map(|xy| {
                    [(xy.xmin, xy.ymin), (xy.xmax, xy.ymax)].map(|(x, y)| Vertex {
                        x,
                        y,
                        ..Default::default()
                    })
                })
                .collect::<Envelope>()
                .xy
                .ok_or(Error::NoGeometries)?,
        };
        let first_resolution = match self.wkid {
            4326 => 360.0 / 512.0,
            3857 => WEB_MERCATOR_WIDTH / 512.0,
            wkid => return Err(Error::Wkid(wkid)),
        };
        let display = DisplayMetadata {
            version: "0.1".to_string(),
            writer: Some(Writer {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            full_extent: EsriEnvelope {
                xmin: full_extent.xmin,
                ymin: full_extent.ymin,
                xmax: full_extent.xmax,
                ymax: full_extent.ymax,
                ..Default::default()
            },
            wkid: Some(self.wkid),
            wkt: None,
            geometry_type: Some(geometry_type),
            has_z,
            has_m,
            json: String::new(),
        };
        let column = |name: &str| ColumnPath::new(vec![GROUP.to_string(), name.to_string()]);
        let points = geometries
            .iter()
            .map(|geometry| match geometry {
                Some(Geometry::Point(Point(vertex))) => *vertex,
                _ => None,
            })
            .collect::<Vec<_>>();

        let (geodisplay, keys) = match geometry_type {
            GeometryType::Point => {
                let z = ZMetadata {
                    display,
                    index_column: column("indexkey"),
                    x_column: column("xcoord"),
                    y_column: column("ycoord"),
                    z_column: has_z.then(|| column("zcoord")),
                    m_column: has_m.then(|| column("mcoord")),
                    coordinate_precision: 20,
                };
                let keys = points
                    .iter()
                    .map(|vertex| match vertex {
                        Some(vertex) => z.index_key(vertex.x, vertex.y),
                        None => 0,
                    })
                    .collect::<Vec<_>>();
                (GeodisplayMetadata::Z(z), keys)
            }
            GeometryType::MultiPoint | GeometryType::Polyline | GeometryType::Polygon => {
                let max_level = MaxLevel::try_from(20)?;
                let first_scale = WEB_MERCATOR_WIDTH * 96.0 * 10_000.0 / (254.0 * 512.0);
                let levels = (0..=16_u8)
                    .step_by(2)
                    .map(|level| {
                        let divisor = 2f64.powi(i32::from(level));
                        let resolution = first_resolution / divisor;
                        MultiscaleLevel {
                            column: column(&format!("esri-multiscale-{level}")),
                            level,
                            resolution,
                            scale: first_scale / divisor,
                            transform: QuantizationTransform {
                                scale: [resolution, resolution, 1.0, 1.0],
                                translate: [0.0; 4],
                            },
                        }
                    })
                    .collect();
                let keys = envelopes
                    .iter()
                    .map(|envelope| match envelope.and_then(|envelope| envelope.xy) {
                        Some(xy) => max_level.index_key(&full_extent, &xy),
                        None => 0,
                    })
                    .collect::<Vec<_>>();
                let xz = XzMetadata {
                    display,
                    max_level,
                    index_column: column("indexkey"),
                    levels,
                    encoding: Encoding::EsriPbf,
                };
                (GeodisplayMetadata::Xz(xz), keys)
            }
            other => return Err(Error::GeometryType(other)),
        };

        let mut order = keys
            .into_iter()
            .enumerate()
            .map(|(row, key)| (key, row))
            .collect::<Vec<_>>();
        order.sort_unstable();
        let rows = order.iter().map(|&(_, row)| row).collect::<Vec<_>>();

        let mut fields = vec![Field::new("indexkey", DataType::Int64, false)];
        let mut arrays = vec![Arc::new(Int64Array::from_iter_values(
            order.iter().map(|&(key, _)| key),
        )) as ArrayRef];
        match &geodisplay {
            GeodisplayMetadata::Xz(xz) => {
                let description = VertexDescription::from(&xz.display);
                let min_length = match geometry_type {
                    GeometryType::MultiPoint => 1,
                    GeometryType::Polyline => 2,
                    _ => 3,
                };
                let width = 2 + usize::from(has_z) + usize::from(has_m);
                for level in &xz.levels {
                    let quantization =
                        Quantization::new(&pbf::Transform::from(&level.transform), description);
                    let mut builder = BinaryBuilder::new();
                    for geometry in rows
                        .iter()
                        .map(|&row| geometries.get(row).and_then(Option::as_ref))
                    {
                        let Some(geometry) = geometry else {
                            builder.append_null();
                            continue;
                        };
                        let simplified = match (has_z || has_m, geometry) {
                            (true, Geometry::Polyline(Polyline(paths))) => {
                                Some(Geometry::Polyline(Polyline(
                                    paths
                                        .simplify(level.resolution)
                                        .ok_or(ToPbfError::UnsupportedGeometry("curve"))?,
                                )))
                            }
                            (true, Geometry::Polygon(polygon)) => {
                                Some(Geometry::Polygon(Polygon {
                                    rings: polygon
                                        .rings
                                        .simplify(level.resolution)
                                        .ok_or(ToPbfError::UnsupportedGeometry("curve"))?,
                                    fill_rule: polygon.fill_rule,
                                }))
                            }
                            _ => None,
                        };
                        let encoded =
                            quantization.encode(simplified.as_ref().unwrap_or(geometry))?;
                        let mut kept = pbf::Geometry::default();
                        let mut offset = 0;
                        for &length in &encoded.lengths {
                            let end = offset + length as usize * width;
                            if length as usize >= min_length {
                                kept.lengths.push(length);
                                kept.coords.extend_from_slice(
                                    encoded.coords.get(offset..end).unwrap_or_default(),
                                );
                            }
                            offset = end;
                        }
                        if kept.lengths.is_empty() {
                            let first = match geometry {
                                Geometry::MultiPoint(points) => points.vertices.get(0),
                                Geometry::Polyline(line) => line.0.vertices.get(0),
                                Geometry::Polygon(polygon) => polygon.rings.vertices.get(0),
                                _ => None,
                            };
                            kept = quantization.encode(&Geometry::Point(Point(first)))?;
                        }
                        builder.append_value(kept.encode_to_vec());
                    }
                    let name = level.column.parts().last().cloned().unwrap_or_default();
                    fields.push(Field::new(name, DataType::Binary, true));
                    arrays.push(Arc::new(builder.finish()));
                }
            }
            GeodisplayMetadata::Z(z) => {
                let sorted = rows
                    .iter()
                    .map(|&row| points.get(row).copied().flatten())
                    .collect::<Vec<_>>();
                let ordinates = [
                    (
                        Some(&z.x_column),
                        (|v: Vertex| Some(v.x)) as fn(Vertex) -> Option<f64>,
                    ),
                    (Some(&z.y_column), |v: Vertex| Some(v.y)),
                    (z.z_column.as_ref(), |v: Vertex| v.z),
                    (z.m_column.as_ref(), |v: Vertex| v.m),
                ];
                for (path, ordinate) in ordinates {
                    let Some(path) = path else {
                        continue;
                    };
                    let values = sorted
                        .iter()
                        .map(|vertex| vertex.and_then(ordinate).unwrap_or(f64::NAN));
                    let name = path.parts().last().cloned().unwrap_or_default();
                    fields.push(Field::new(name, DataType::Float64, false));
                    arrays.push(Arc::new(Float64Array::from_iter_values(values)));
                }
            }
        }

        let indices = UInt64Array::from_iter_values(rows.iter().map(|&row| row as u64));
        let mut columns = batch
            .columns()
            .iter()
            .map(|column| take(column, &indices, None))
            .collect::<Result<Vec<_>, _>>()?;
        let fields = Fields::from(fields);
        columns.push(Arc::new(StructArray::try_new(
            fields.clone(),
            arrays,
            None,
        )?));
        let schema = batch.schema();
        let schema = Schema::new_with_metadata(
            schema
                .fields()
                .iter()
                .cloned()
                .chain([Arc::new(Field::new(GROUP, DataType::Struct(fields), false))])
                .collect::<Fields>(),
            schema.metadata().clone(),
        );
        Ok(Optimized {
            batch: RecordBatch::try_new(Arc::new(schema), columns)?,
            geodisplay,
        })
    }
}

impl Optimized {
    pub fn properties(&self) -> Result<WriterPropertiesBuilder, Error> {
        let json = serde_json::to_string(&self.geodisplay)?;
        Ok(self.geodisplay.index_columns().into_iter().fold(
            WriterProperties::builder()
                .set_key_value_metadata(Some(vec![KeyValue::new(GROUP.to_string(), json)]))
                .set_statistics_enabled(EnabledStatistics::Page)
                .set_max_row_group_row_count(Some(128 * 1024))
                .set_max_row_group_bytes(Some(128 * 1024 * 1024)),
            |builder, path| builder.set_column_dictionary_enabled(path.clone(), false),
        ))
    }
}

#[cfg(test)]
mod tests;
