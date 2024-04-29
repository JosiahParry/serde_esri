use crate::deserialize_sr;
use extendr_api::prelude::*;
use serde_esri::{
    features::{Feature, FeatureSet},
    geometry::EsriGeometry,
    spatial_reference::SpatialReference,
};

use crate::{sfc::*, sfg::SfgPoint};

impl SfcPoint {
    /// Consume an SfcPoint to return a vector of Fetaures
    pub fn as_features<const N: usize>(self) -> Result<Vec<Feature<N>>> {
        let feats = self
            .0
            .into_iter()
            .map(|(_, feat)| {
                let inner = Doubles::try_from(feat).expect("doubles vector");
                let geom = SfgPoint(inner)
                    .as_point()
                    .expect("correct length of doubles vector");

                Feature {
                    geometry: Some(EsriGeometry::Point(geom)),
                    attributes: None,
                }
            })
            .collect::<Vec<_>>();
        Ok(feats)
    }
    /// Consume an SfcPoint to create a FeatureSet<N>
    /// A spatial reference should be created and passed in from the `crs` attribute
    /// of the sfc object. For points, it is safe to ignore the `<N>` constant, I believe.
    pub fn as_featureset<const N: usize>(self, sr: Option<SpatialReference>) -> FeatureSet<N> {
        let feats = self.as_features().expect("Features to be created");
        FeatureSet {
            objectIdFieldName: None,
            globalIdFieldName: None,
            displayFieldName: None,
            spatialReference: sr,
            geometryType: Some("esriGeometryPoint".into()),
            features: feats,
            fields: None,
        }
    }
}

#[extendr]
fn sfc_point_features_2d(x: List) -> String {
    let sfc = SfcPoint(x);
    let features = sfc.as_features::<2>().unwrap();
    serde_json::to_string(&features).unwrap()
}

#[extendr]
fn sfc_point_features_3d(x: List) -> String {
    let sfc = SfcPoint(x);
    let features = sfc.as_features::<3>().unwrap();
    serde_json::to_string(&features).unwrap()
}

#[extendr]
// TODO Handle CRS
fn sfc_point_featureset_2d(x: List, sr: Robj) -> String {
    let sfc = SfcPoint(x);
    // This should be part of the R library
    let crs = deserialize_sr(&sr);
    let featureset = sfc.as_featureset::<2>(crs);
    serde_json::to_string(&featureset).unwrap()
}

#[extendr]
// TODO Handle CRS
fn sfc_point_featureset_3d(x: List, sr: Robj) -> String {
    let sfc = SfcPoint(x);
    let crs = deserialize_sr(&sr);
    let featureset = sfc.as_featureset::<3>(crs);
    serde_json::to_string(&featureset).unwrap()
}

extendr_module! {
    mod point;
    fn sfc_point_features_2d;
    fn sfc_point_features_3d;
    fn sfc_point_featureset_2d;
    fn sfc_point_featureset_3d;
}
