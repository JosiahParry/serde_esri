use crate::{deserialize_sr, sfc::*, sfg::SfgLineString, to::AsEsriGeometry};
use extendr_api::prelude::*;
use serde_esri::{
    features::Feature, features::FeatureSet, geometry::EsriGeometry, geometry::EsriPolyline,
    spatial_reference::SpatialReference,
};

impl SfcLineString {
    pub fn as_features_2d(self) -> Result<Vec<Feature<2>>> {
        let lstrs = self
            .0
            .into_iter()
            .map(|(_, lstr)| {
                let lstr_list = RMatrix::try_from(lstr);
                let lstr_list = match lstr_list {
                    Ok(lstr_list) => {
                        let sfg = SfgLineString(lstr_list);
                        let lstr: Option<EsriPolyline<2>> = sfg.as_polyline();
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
                    attributes: None,
                }
            })
            .collect::<Vec<_>>();

        Ok(lstrs)
    }

    pub fn as_features_3d(self) -> Result<Vec<Feature<3>>> {
        let lstrs = self
            .0
            .into_iter()
            .map(|(_, lstr)| {
                let lstr_list = RMatrix::try_from(lstr);
                let lstr_list = match lstr_list {
                    Ok(lstr_list) => {
                        let sfg = SfgLineString(lstr_list);
                        let lstr: Option<EsriPolyline<3>> = sfg.as_polyline();
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
                    attributes: None,
                }
            })
            .collect::<Vec<_>>();

        Ok(lstrs)
    }

    pub fn as_featureset_2d(self, sr: Option<SpatialReference>) -> FeatureSet<2> {
        let feats = self.as_features_2d().expect("Features to be created");
        FeatureSet {
            objectIdFieldName: None,
            globalIdFieldName: None,
            displayFieldName: None,
            spatialReference: sr,
            geometryType: Some("esriGeometryLineString".into()),
            features: feats,
            fields: None,
        }
    }

    pub fn as_featureset_3d(self, sr: Option<SpatialReference>) -> FeatureSet<3> {
        let feats = self.as_features_3d().expect("Features to be created");
        FeatureSet {
            objectIdFieldName: None,
            globalIdFieldName: None,
            displayFieldName: None,
            spatialReference: sr,
            geometryType: Some("esriGeometryLineString".into()),
            features: feats,
            fields: None,
        }
    }
}

#[extendr]
fn sfc_linestring_features_2d(x: List) -> String {
    let res = SfcLineString(x).as_features_2d().unwrap();
    serde_json::to_string(&res).unwrap()
}

#[extendr]
fn sfc_linestring_features_3d(x: List) -> String {
    let res = SfcLineString(x).as_features_3d().unwrap();
    serde_json::to_string(&res).unwrap()
}

#[extendr]
fn sfc_linestring_featureset_2d(x: List, sr: Robj) -> String {
    let sfc = SfcLineString(x);
    let crs = deserialize_sr(&sr);
    let featureset = sfc.as_featureset_2d(crs);
    serde_json::to_string(&featureset).unwrap()
}

#[extendr]
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
