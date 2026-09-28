//! Converts query results into Arrow record batches with a GeoArrow geometry column.

mod geometry;

use crate::{
    convert::FromPbfError,
    feature_collection_p_buffer::{self as pbf, query_result::Results},
    FeatureCollectionPBuffer,
};
use arrow_array::{ArrayRef, RecordBatch, RecordBatchOptions};
use arrow_schema::{ArrowError, Field, Schema};
use geoarrow_schema::error::GeoArrowError;
use geometry::GeometryColumnBuilder;
use serde_esri::{
    arrow_compat::{AttributeColumn, ToArrowError},
    features::EsriValue,
};
use std::{collections::HashSet, sync::Arc};

#[derive(Debug)]
pub enum PbfToArrowError {
    Pbf(FromPbfError),
    Arrow(ToArrowError),
}

impl std::fmt::Display for PbfToArrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PbfToArrowError::Pbf(e) => write!(f, "{e}"),
            PbfToArrowError::Arrow(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for PbfToArrowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PbfToArrowError::Pbf(e) => Some(e),
            PbfToArrowError::Arrow(e) => Some(e),
        }
    }
}

impl From<FromPbfError> for PbfToArrowError {
    fn from(e: FromPbfError) -> Self {
        PbfToArrowError::Pbf(e)
    }
}

impl From<ToArrowError> for PbfToArrowError {
    fn from(e: ToArrowError) -> Self {
        PbfToArrowError::Arrow(e)
    }
}

impl From<GeoArrowError> for PbfToArrowError {
    fn from(e: GeoArrowError) -> Self {
        PbfToArrowError::Arrow(e.into())
    }
}

impl From<ArrowError> for PbfToArrowError {
    fn from(e: ArrowError) -> Self {
        PbfToArrowError::Arrow(e.into())
    }
}

/// Each value goes straight into its field's column, in field order; fields repeating an
/// earlier name, and geometry, blob, and raster fields, have no column.
impl TryFrom<pbf::FeatureResult> for RecordBatch {
    type Error = PbfToArrowError;

    fn try_from(result: pbf::FeatureResult) -> Result<Self, Self::Error> {
        let rows = result.features.len();
        let mut geometry = GeometryColumnBuilder::new(&result)?;
        let mut seen = HashSet::new();
        let mut columns: Vec<Option<(String, AttributeColumn)>> = result
            .fields
            .iter()
            .map(|field| {
                let column = AttributeColumn::new(&field.field_type().into(), rows)?;
                seen.insert(field.name.as_str())
                    .then(|| (field.name.clone(), column))
            })
            .collect();

        for feature in result.features {
            let mut values = feature.attributes.into_iter();
            for column in &mut columns {
                let value = values.next().map(EsriValue::from).unwrap_or_default();
                if let Some((_, column)) = column {
                    column.push(&value);
                }
            }
            if let Some(geometry) = geometry.as_mut() {
                geometry.push(feature.compressed_geometry)?;
            }
        }

        let (mut fields, mut arrays): (Vec<Field>, Vec<ArrayRef>) = columns
            .into_iter()
            .flatten()
            .map(|(name, column)| {
                let array = column.finish();
                (Field::new(name, array.data_type().clone(), true), array)
            })
            .unzip();
        if let Some(geometry) = geometry {
            let (field, array) = geometry.finish();
            fields.push(field);
            arrays.push(array);
        }
        let options = RecordBatchOptions::new().with_row_count(Some(rows));
        Ok(RecordBatch::try_new_with_options(
            Arc::new(Schema::new(fields)),
            arrays,
            &options,
        )?)
    }
}

impl TryFrom<FeatureCollectionPBuffer> for RecordBatch {
    type Error = PbfToArrowError;

    fn try_from(collection: FeatureCollectionPBuffer) -> Result<Self, Self::Error> {
        match collection.query_result.and_then(|q| q.results) {
            Some(Results::FeatureResult(result)) => RecordBatch::try_from(result),
            _ => Err(FromPbfError::NotFeatureResult.into()),
        }
    }
}

#[cfg(test)]
mod tests;
