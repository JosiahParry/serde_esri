use crate::de_array::arrays;
use crate::spatial_reference::SpatialReference;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize, Serialize, Debug)]
pub struct EsriCoord<const N: usize>(#[serde(with = "arrays")] pub [f64; N]);

#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct EsriPoint {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub m: Option<f64>,
    pub spatialReference: Option<SpatialReference>,
}

#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct EsriMultiPoint<const N: usize> {
    pub hasZ: Option<bool>,
    pub hasM: Option<bool>,
    pub points: Vec<EsriCoord<N>>,
    pub spatialReference: Option<SpatialReference>,
}

#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct EsriPolyline<const N: usize> {
    pub hasZ: Option<bool>,
    pub hasM: Option<bool>,
    pub paths: Vec<EsriCoord<N>>,
    pub spatialReference: Option<SpatialReference>,
}

#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct EsriPolygon<const N: usize> {
    pub hasZ: Option<bool>,
    pub hasM: Option<bool>,
    pub rings: Vec<Vec<EsriCoord<N>>>,
    pub spatialReference: Option<SpatialReference>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum EsriGeometry<const N: usize> {
    Point(EsriPoint),
    MultiPoint(EsriMultiPoint<N>),
    Polygon(EsriPolygon<N>),
    Polyline(EsriPolyline<N>),
}

impl<const N: usize> EsriGeometry<N> {
    pub fn to_string(&self) -> String {
        let geo_type = match self {
            EsriGeometry::Point(_) => "esriGeometryPoint",
            EsriGeometry::MultiPoint(_) => "esriGeometryMultiPoint",
            EsriGeometry::Polygon(_) => "esriGeometryPolygon",
            EsriGeometry::Polyline(_) => "esriGeometryPolyline",
        };

        geo_type.into()
    }
}

// esriGeometryPoint | esriGeometryMultipoint | esriGeometryPolyline | esriGeometryPolygon | esriGeometryEnvelope.
