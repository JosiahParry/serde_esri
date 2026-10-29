use crate::{
    error::Error,
    metadata::{ColumnLookup, GeodisplayMetadata},
    plan::{MinMax, Plan},
    query::Query,
    shape::ShapeColumn,
    xz::XzRanges,
};
use arrow_array::{cast::AsArray, types::Int64Type, BooleanArray, RecordBatch};
use arrow_schema::{Schema, SchemaRef};
use arrow_select::filter::filter_record_batch;
use bytes::Bytes;
use parquet::{
    arrow::{
        arrow_reader::{ArrowPredicateFn, ArrowReaderMetadata, RowFilter},
        push_decoder::{ParquetPushDecoder, ParquetPushDecoderBuilder},
        ProjectionMask,
    },
    DecodeResult,
};
use serde_esri::enginex::geometry::Envelope2D;
use std::{collections::BTreeSet, ops::Range, sync::Arc};

pub struct Decoder {
    decoder: ParquetPushDecoder,
    schema: SchemaRef,
    columns: Vec<String>,
    shape: Option<ShapeColumn>,
}

impl Decoder {
    pub fn try_new(metadata: ArrowReaderMetadata, query: &Query) -> Result<Self, Error> {
        let geodisplay = GeodisplayMetadata::try_from(metadata.metadata().file_metadata())?;
        let descriptor = metadata.parquet_schema();
        let arrow_schema = metadata.schema();
        let shape = ShapeColumn::try_new(&geodisplay, query)?;

        let hidden = geodisplay
            .index_columns()
            .into_iter()
            .filter_map(|path| path.parts().first().map(String::as_str))
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
        if let Some(shape) = &shape {
            for path in shape.paths() {
                leaves.insert(path.leaf(descriptor)?);
            }
            fields.push(Arc::new(shape.field().clone()));
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
                    let index_leaf = xz.index_column.leaf(descriptor)?;
                    let plan = Plan::new(&parquet_metadata, |row_group, index| {
                        i64::rows(
                            &parquet_metadata,
                            row_group,
                            index,
                            index_leaf,
                            |[min, max]| ranges.overlaps(min, max),
                        )
                    });
                    let path = xz.index_column.clone();
                    let predicate = ArrowPredicateFn::new(
                        ProjectionMask::leaves(descriptor, [index_leaf]),
                        move |batch: RecordBatch| {
                            let values = path
                                .column(&batch)?
                                .as_primitive_opt::<Int64Type>()
                                .ok_or_else(|| Error::ColumnType {
                                    column: path.to_string(),
                                    expected: "64-bit integer",
                                })?;
                            Ok(values
                                .iter()
                                .map(|xz| Some(xz.is_some_and(|xz| ranges.overlaps(xz, xz))))
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
                                f64::rows(&parquet_metadata, row_group, index, x, |[lo, hi]| {
                                    lo <= e.xmax && hi >= e.xmin
                                })
                                .intersection(&f64::rows(
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
                            let xs = x_path.doubles(&batch)?;
                            let ys = y_path.doubles(&batch)?;
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
            shape,
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
            let keep = match &self.shape {
                Some(shape) => {
                    let (shapes, keep) = shape.build(&batch)?;
                    columns.push(shapes);
                    keep
                }
                None => None,
            };
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

#[cfg(test)]
pub(crate) mod tests;
