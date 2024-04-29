use crate::{sfc::*, sfg::SfgMultiPoint, to::AsEsriGeometry};

use extendr_api::prelude::*;
use serde_esri::{
    features::{Feature, FeatureSet},
    geometry::{EsriGeometry, EsriMultiPoint},
    spatial_reference::SpatialReference,
};

impl SfcMultiPoint {
    pub fn as_features_2d(self) -> Result<Vec<Feature<2>>> {
        let mpnts = self
            .0
            .into_iter()
            .map(|(_, pnt)| {
                let pnt_mat = RMatrix::try_from(pnt);
                let pnt_mat = match pnt_mat {
                    Ok(pnt_mat) => {
                        let sfg = SfgMultiPoint(pnt_mat);
                        let mpnt: Option<EsriMultiPoint<2>> = sfg.as_multipoint();
                        mpnt.unwrap()
                    }
                    Err(_) => EsriMultiPoint {
                        hasZ: Some(false),
                        hasM: Some(false),
                        points: vec![],
                        spatialReference: None,
                    },
                };

                Feature::<2> {
                    geometry: Some(EsriGeometry::MultiPoint(pnt_mat)),
                    attributes: None,
                }
            })
            .collect::<Vec<_>>();

        Ok(mpnts)
    }

    pub fn as_features_3d(self) -> Result<Vec<Feature<3>>> {
        let mpnts = self
            .0
            .into_iter()
            .map(|(_, pnt)| {
                let pnt_mat = RMatrix::try_from(pnt);
                let pnt_mat = match pnt_mat {
                    Ok(pnt_mat) => {
                        let sfg = SfgMultiPoint(pnt_mat);
                        let mpnt: Option<EsriMultiPoint<3>> = sfg.as_multipoint();
                        mpnt.unwrap()
                    }
                    Err(_) => EsriMultiPoint {
                        hasZ: Some(false),
                        hasM: Some(false),
                        points: vec![],
                        spatialReference: None,
                    },
                };

                Feature::<3> {
                    geometry: Some(EsriGeometry::MultiPoint(pnt_mat)),
                    attributes: None,
                }
            })
            .collect::<Vec<_>>();

        Ok(mpnts)
    }

    // TODO: Implement as_features_4d but not supported in sf
}

#[extendr]
fn as_multipoint_features_2d(x: List) -> String {
    let res = SfcMultiPoint(x).as_features_2d().unwrap();
    serde_json::to_string(&res).unwrap()
}

extendr_module! {
    mod multipoint;
    fn as_multipoint_features_2d;
}
