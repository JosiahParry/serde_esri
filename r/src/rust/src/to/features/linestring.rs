use crate::{deserialize_sr, sfc::*, sfg::SfgLineString, to::AsEsriGeometry};
use extendr_api::prelude::*;
use serde_esri::{
    features::Feature, features::FeatureSet, geometry::EsriGeometry, geometry::EsriPolyline,
    spatial_reference::SpatialReference,
};

use serde_json::Map;

impl SfcLineString {
    pub fn as_features_2d(self, sr: Option<SpatialReference>) -> Result<Vec<Feature<2>>> {
        let lstrs = self
            .0
            .into_iter()
            .map(|(_, lstr)| {
                let lstr_list = RMatrix::try_from(lstr);
                let lstr_list = match lstr_list {
                    Ok(lstr_list) => {
                        let sfg = SfgLineString(lstr_list);
                        let lstr: Option<EsriPolyline<2>> = sfg.as_polyline(sr.clone());
                        lstr.unwrap()
                    }
                    Err(_) => EsriPolyline {
                        hasZ: Some(false),
                        hasM: Some(false),
                        paths: vec![],
                        spatialReference: None,
                    },
                };

                Feature::<2> {
                    geometry: Some(EsriGeometry::Polyline(lstr_list)),
                    attributes: Some(Map::default()),
                }
            })
            .collect::<Vec<_>>();

        Ok(lstrs)
    }

    pub fn as_features_3d(self, sr: Option<SpatialReference>) -> Result<Vec<Feature<3>>> {
        let lstrs = self
            .0
            .into_iter()
            .map(|(_, lstr)| {
                let lstr_list = RMatrix::try_from(lstr);
                let lstr_list = match lstr_list {
                    Ok(lstr_list) => {
                        let sfg = SfgLineString(lstr_list);
                        let lstr: Option<EsriPolyline<3>> = sfg.as_polyline(sr.clone());
                        lstr.unwrap()
                    }
                    Err(_) => EsriPolyline {
                        hasZ: Some(false),
                        hasM: Some(false),
                        paths: vec![],
                        spatialReference: None,
                    },
                };

                Feature::<3> {
                    geometry: Some(EsriGeometry::Polyline(lstr_list)),
                    attributes: Some(Map::default()),
                }
            })
            .collect::<Vec<_>>();

        Ok(lstrs)
    }

    pub fn as_featureset_2d(self, sr: Option<SpatialReference>) -> FeatureSet<2> {
        let feats = self.as_features_2d(None).expect("Features to be created");
        FeatureSet {
            objectIdFieldName: None,
            globalIdFieldName: None,
            displayFieldName: None,
            spatialReference: sr,
            geometryType: Some("esriGeometryLineString".into()),
            features: feats,
            fields: None,
            hasM: None,
            hasZ: None,
        }
    }

    pub fn as_featureset_3d(self, sr: Option<SpatialReference>) -> FeatureSet<3> {
        let feats = self.as_features_3d(None).expect("Features to be created");
        FeatureSet {
            objectIdFieldName: None,
            globalIdFieldName: None,
            displayFieldName: None,
            spatialReference: sr,
            geometryType: Some("esriGeometryLineString".into()),
            features: feats,
            fields: None,
            // TODO parameterize this??
            // how can we propagate the hasZ and M forward/
            hasM: None,
            hasZ: Some(true),
        }
    }
}

#[extendr]
/// Create an EsriJSON feature array
/// @param an sfc geometry vector
/// @export
/// @rdname features
fn sfc_linestring_features_2d(x: List, sr: Robj) -> String {
    let sr = deserialize_sr(&sr);
    let res = SfcLineString(x).as_features_2d(sr).unwrap();
    serde_json::to_string(&res).unwrap()
}

#[extendr]
/// @export
/// @rdname features
fn sfc_linestring_features_3d(x: List, sr: Robj) -> String {
    let sr = deserialize_sr(&sr);
    let res = SfcLineString(x).as_features_3d(sr).unwrap();
    serde_json::to_string(&res).unwrap()
}

#[extendr]
/// @export
/// @rdname featureset
fn sfc_linestring_featureset_2d(x: List, sr: Robj) -> String {
    let sfc = SfcLineString(x);
    let crs = deserialize_sr(&sr);
    let featureset = sfc.as_featureset_2d(crs);
    serde_json::to_string(&featureset).unwrap()
}

#[extendr]
/// @export
/// @rdname featureset
fn sfc_linestring_featureset_3d(x: List, sr: Robj) -> String {
    let sfc = SfcLineString(x);
    let crs = deserialize_sr(&sr);
    let featureset = sfc.as_featureset_3d(crs);
    serde_json::to_string(&featureset).unwrap()
}

extendr_module! {
    mod linestring;
    fn sfc_linestring_features_2d;
    fn sfc_linestring_features_3d;
    fn sfc_linestring_featureset_2d;
    fn sfc_linestring_featureset_3d;
}
