use crate::{deserialize_sr, sfc::*, sfg::SfgPolygon, to::AsEsriGeometry};
use extendr_api::prelude::*;
use serde_esri::{
    features::Feature, features::FeatureSet, geometry::EsriGeometry, geometry::EsriPolygon,
    spatial_reference::SpatialReference,
};

use serde_json::Map;

impl SfcPolygon {
    pub fn as_features_2d(self) -> Result<Vec<Feature<2>>> {
        let lstrs = self
            .0
            .into_iter()
            .map(|(_, lstr)| {
                let lstr_list = List::try_from(lstr);
                let lstr_list = match lstr_list {
                    Ok(lstr_list) => {
                        let sfg = SfgPolygon(lstr_list);
                        let lstr: Option<EsriPolygon<2>> = sfg.as_polygon();
                        lstr.unwrap()
                    }
                    Err(_) => EsriPolygon {
                        hasZ: Some(false),
                        hasM: Some(false),
                        rings: vec![],
                        spatialReference: None,
                    },
                };

                Feature::<2> {
                    geometry: Some(EsriGeometry::Polygon(lstr_list)),
                    attributes: Some(Map::default()),
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
                let lstr_list = List::try_from(lstr);
                let lstr_list = match lstr_list {
                    Ok(lstr_list) => {
                        let sfg = SfgPolygon(lstr_list);
                        let lstr: Option<EsriPolygon<3>> = sfg.as_polygon();
                        lstr.unwrap()
                    }
                    Err(_) => EsriPolygon {
                        hasZ: Some(false),
                        hasM: Some(false),
                        rings: vec![],
                        spatialReference: None,
                    },
                };

                Feature::<3> {
                    geometry: Some(EsriGeometry::Polygon(lstr_list)),
                    attributes: Some(Map::default()),
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
            geometryType: Some("esriGeometryPolygon".into()),
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
            geometryType: Some("esriGeometryPolygon".into()),
            features: feats,
            fields: None,
        }
    }
}

#[extendr]
fn sfc_polygon_features_2d(x: List) -> String {
    let res = SfcPolygon(x).as_features_2d().unwrap();
    serde_json::to_string(&res).unwrap()
}

#[extendr]
fn sfc_polygon_features_3d(x: List) -> String {
    let res = SfcPolygon(x).as_features_3d().unwrap();
    serde_json::to_string(&res).unwrap()
}

#[extendr]
fn sfc_polygon_featureset_2d(x: List, sr: Robj) -> String {
    let sfc = SfcPolygon(x);
    let crs = deserialize_sr(&sr);
    let featureset = sfc.as_featureset_2d(crs);
    serde_json::to_string(&featureset).unwrap()
}

#[extendr]
fn sfc_polygon_featureset_3d(x: List, sr: Robj) -> String {
    let sfc = SfcPolygon(x);
    let crs = deserialize_sr(&sr);
    let featureset = sfc.as_featureset_3d(crs);
    serde_json::to_string(&featureset).unwrap()
}

extendr_module! {
    mod polygon;
    fn sfc_polygon_features_2d;
    fn sfc_polygon_features_3d;
    fn sfc_polygon_featureset_2d;
    fn sfc_polygon_featureset_3d;
}
