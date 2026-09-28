//! Converts query results into Arrow record batches with a GeoArrow geometry column.

use crate::{
    convert::FromPbfError,
    feature_collection_p_buffer::{self as pbf, query_result::Results},
    FeatureCollectionPBuffer,
};
use arrow_array::RecordBatch;
use serde_esri::{arrow_compat::ToArrowError, features::FeatureSet};

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

/// Goes through the `FeatureSet` whose width `hasZ` and `hasM` call for.
impl TryFrom<pbf::FeatureResult> for RecordBatch {
    type Error = PbfToArrowError;

    fn try_from(result: pbf::FeatureResult) -> Result<Self, Self::Error> {
        let batch = match (result.has_z, result.has_m) {
            (false, false) => RecordBatch::try_from(&FeatureSet::<2>::try_from(result)?)?,
            (true, true) => RecordBatch::try_from(&FeatureSet::<4>::try_from(result)?)?,
            _ => RecordBatch::try_from(&FeatureSet::<3>::try_from(result)?)?,
        };
        Ok(batch)
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
