//! Struct representations of [Esri JSON Geometry Objects](https://developers.arcgis.com/documentation/common-data-types/geometry-objects.htm)
//!
//! The spatial reference of a geometry object is not always provided.
//! As such it's encoded as an `Option<SpatialReference>` to handle the cases where it may
//! or may not be provided.
use crate::de_array::arrays;
use crate::spatial_reference::SpatialReference;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Represents a single coordinate used in the creation of `EsriPolygon` and `EsriPolyline`s.
///
/// It requires a valid `N` of values per coordinate. Should always be one of
/// `2`, `3`, or `4` in the case of XY, XYZ, or XYZM coordinates.
#[derive(Clone, Deserialize, Serialize, Debug)]
// From discord: https://discord.com/channels/273534239310479360/273541522815713281/1175836081091006484
// Motivation: converting from this to Arrow
// "you could go from a Box<[f64; 4]> to a Vec<f64> without copying or reallocating,
// because in that case the allocation already exists"
pub struct EsriCoord<const N: usize>(#[serde(with = "arrays")] pub [f64; N]);

/// An `esriGeometryPoint` with fields x, y, z, and m. x and y are both required.
#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EsriPoint {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub m: Option<f64>,
    pub spatialReference: Option<SpatialReference>,
}

/// An `esriGeometryMultipoint` defined by a vector of `EsriCoord`s.
///
/// `<N>` parameter should be equal to `2 + hasZ + hasM`. There are no
/// checks on the const value. If an incorrect value is provided, expect
/// a `panic!`.
#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EsriMultiPoint<const N: usize> {
    pub hasZ: Option<bool>,
    pub hasM: Option<bool>,
    pub points: Vec<EsriCoord<N>>,
    pub spatialReference: Option<SpatialReference>,
}

// Implement iterators for EsriMultiPoint struct
// this is mostly copy pasta from ChatGPT. Not _Too_ sure
// what is happening in here
pub struct EsriMultiPointIterator<'a, const N: usize> {
    points_iter: std::slice::Iter<'a, EsriCoord<N>>
}

impl<const N: usize> EsriMultiPoint<N> {
    pub fn iter(&self) -> EsriMultiPointIterator<N> {
        EsriMultiPointIterator {
            points_iter: self.points.iter(),
        }
    }
}

impl<'a, const N: usize> Iterator for EsriMultiPointIterator<'a, N> {
    type Item = &'a EsriCoord<N>; // Define the associated type 'Item'

    fn next(&mut self) -> Option<Self::Item> {
        self.points_iter.next()
    }
}

impl<'a, const N: usize> IntoIterator for &'a EsriMultiPoint<N> {
    type Item = &'a EsriCoord<N>; // Define the associated type 'Item' for IntoIterator
    type IntoIter = EsriMultiPointIterator<'a, N>; // Define the associated type 'IntoIter'

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, const N: usize> ExactSizeIterator for EsriMultiPointIterator<'a, N> {
    fn len(&self) -> usize {
        self.points_iter.len()
    }
}

// I'm going to try creating an EsriLineString struct
// which is going to be used for internally for polyline and polygon
// this is so that i can implement linestring trait for geoarrow
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EsriLineString<const N: usize>(Vec<EsriCoord<N>>);

/// An `esriGeometryPolyline` defined by a vector of `Vec<EsriCoord<N>>`.
///
/// Each inner vector should be a single linestring.
///
/// `<N>` parameter should be equal to `2 + hasZ + hasM`. There are no
/// checks on the const value. If an incorrect value is provided, expect
/// a `panic!`.
#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EsriPolyline<const N: usize> {
    pub hasZ: Option<bool>,
    pub hasM: Option<bool>,
    pub paths: Vec<EsriLineString<N>>,
    pub spatialReference: Option<SpatialReference>,
}

/// An `esriGeometryPolygon` defined by a `Vec<Vec<EsriCoord<N>>>`
///
/// Each inner vector should be a single linear ring. The first `Vec<EsriCoord<N>>`
/// represents the exterior ring. Subsequent ones are interior rings. No checking
/// of widing occurs.
///
/// `<N>` parameter should be equal to `2 + hasZ + hasM`. There are no
/// checks on the const value. If an incorrect value is provided, expect
/// a `panic!`.
#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EsriPolygon<const N: usize> {
    pub hasZ: Option<bool>,
    pub hasM: Option<bool>,
    pub rings: Vec<Vec<EsriCoord<N>>>,
    pub spatialReference: Option<SpatialReference>,
}

/// An enum of all valid geometry types. At present this does not include `esriGeometryEnvelope`
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum EsriGeometry<const N: usize> {
    Point(EsriPoint),
    MultiPoint(EsriMultiPoint<N>),
    Polygon(EsriPolygon<N>),
    Polyline(EsriPolyline<N>),
}

// Completed: esriGeometryPoint | esriGeometryMultipoint | esriGeometryPolyline | esriGeometryPolygon |
// TODO: esriGeometryEnvelope.
