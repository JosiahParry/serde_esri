use extendr_api::deserializer::from_robj;
use extendr_api::prelude::*;
use serde_esri::{
    features::{Feature, FeatureSet},
    geometry::EsriGeometry,
    spatial_reference::SpatialReference,
};
use serde_json::{Map, Value};

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