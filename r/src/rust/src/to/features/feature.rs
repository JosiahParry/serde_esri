use extendr_api::prelude::*;
use serde_esri::{
    features::FeatureSet, geometry::EsriGeometry, spatial_reference::SpatialReference,
};

use crate::sfg::SfgPoint;

pub trait AsFeatures<const N: usize> {
    type Output;
    fn as_features(self) -> Result<Vec<Self::Output>>;
    fn as_featureset(self, sr: Option<SpatialReference>) -> FeatureSet<N>;
}
