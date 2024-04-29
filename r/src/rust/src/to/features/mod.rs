mod multipoint;
mod point;

use extendr_api::prelude::*;

extendr_module! {
    mod features;
    use point;
    use multipoint;
}
// pub trait AsFeatures<const N: usize> {
//     type Output;
//     fn as_features(self) -> Result<Vec<Self::Output>>;
//     fn as_featureset(self, sr: Option<SpatialReference>) -> FeatureSet<N>;
// }

// use extendr_api::deserializer::from_robj;
// use extendr_api::prelude::*;
// use serde_esri::{
//     features::{Feature, FeatureSet},
//     geometry::EsriGeometry,
//     spatial_reference::SpatialReference,
// };

// use serde_json::{Map, Value};

// use crate::{sfc::*, sfg::SfgPoint};

// trait AsFeatures<const N: usize> {
//     type Output;
//     fn as_features(self) -> Result<Vec<Self::Output>>;
//     fn as_featureset(self, sr: Option<SpatialReference>) -> FeatureSet<N>;
// }
