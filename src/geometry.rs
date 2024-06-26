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
#[skip_serializing_none]
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct EsriCoord<const N: usize>(#[serde(with = "arrays")] pub [f64; N]);

/// An `esriGeometryPoint` with fields x, y, z, and m. x and y are both required.
#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
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
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
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
    points_iter: std::slice::Iter<'a, EsriCoord<N>>,
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

/// This struct is used strictly for representing the internal LineStrings
/// for the `EsriPolygon` and `EsriPolyline` structs. They do not represent
/// any Esri JSON geometry objects.
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct EsriLineString<const N: usize>(pub Vec<EsriCoord<N>>);

pub struct EsriLineStringIterator<'a, const N: usize> {
    iter: std::slice::Iter<'a, EsriCoord<N>>,
}

impl<'a, const N: usize> Iterator for EsriLineStringIterator<'a, N> {
    type Item = &'a EsriCoord<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, const N: usize> ExactSizeIterator for EsriLineStringIterator<'a, N> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<const N: usize> IntoIterator for EsriLineString<N> {
    type Item = EsriCoord<N>;
    type IntoIter = std::vec::IntoIter<EsriCoord<N>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const N: usize> EsriLineString<N> {
    pub fn iter(&self) -> EsriLineStringIterator<N> {
        EsriLineStringIterator {
            iter: self.0.iter(),
        }
    }
}
/// An `esriGeometryPolyline` defined by a vector of `Vec<EsriCoord<N>>`.
///
/// Each inner vector should be a single linestring.
///
/// `<N>` parameter should be equal to `2 + hasZ + hasM`. There are no
/// checks on the const value. If an incorrect value is provided, expect
/// a `panic!`.
#[skip_serializing_none]
#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct EsriPolyline<const N: usize> {
    pub hasZ: Option<bool>,
    pub hasM: Option<bool>,
    pub paths: Vec<EsriLineString<N>>,
    pub spatialReference: Option<SpatialReference>,
}

pub struct EsriPolylineIterator<'a, const N: usize> {
    pub paths_iter: std::slice::Iter<'a, EsriLineString<N>>,
}

impl<'a, const N: usize> Iterator for EsriPolylineIterator<'a, N> {
    type Item = &'a EsriLineString<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.paths_iter.next()
    }
}

impl<'a, const N: usize> ExactSizeIterator for EsriPolylineIterator<'a, N> {
    fn len(&self) -> usize {
        self.paths_iter.len()
    }
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
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct EsriPolygon<const N: usize> {
    pub hasZ: Option<bool>,
    pub hasM: Option<bool>,
    pub rings: Vec<EsriLineString<N>>,
    pub spatialReference: Option<SpatialReference>,
}

pub struct EsriPolygonIterator<'a, const N: usize> {
    pub paths_iter: std::slice::Iter<'a, EsriLineString<N>>,
}

impl<'a, const N: usize> Iterator for EsriPolygonIterator<'a, N> {
    type Item = &'a EsriLineString<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.paths_iter.next()
    }
}

impl<'a, const N: usize> ExactSizeIterator for EsriPolygonIterator<'a, N> {
    fn len(&self) -> usize {
        self.paths_iter.len()
    }
}

/// An enum of all valid geometry types. At present this does not include `esriGeometryEnvelope`
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum EsriGeometry<const N: usize> {
    Point(EsriPoint),
    MultiPoint(EsriMultiPoint<N>),
    Polygon(EsriPolygon<N>),
    Polyline(EsriPolyline<N>),
    Envelope(EsriEnvelope),
}

impl<const N: usize> EsriGeometry<N> {
    /// Returns a point if possible
    pub fn as_point(self) -> Option<EsriPoint> {
        match self {
            EsriGeometry::Point(p) => Some(p),
            _ => None,
        }
    }

    /// Returns a multipoint if possible
    pub fn as_multipoint(self) -> Option<EsriMultiPoint<N>> {
        match self {
            EsriGeometry::MultiPoint(p) => Some(p),
            _ => None,
        }
    }

    /// Returns a polyline if possible
    pub fn as_polyline(self) -> Option<EsriPolyline<N>> {
        match self {
            EsriGeometry::Polyline(pl) => Some(pl),
            _ => None,
        }
    }

    /// Returns a polygon if possible
    pub fn as_polygon(self) -> Option<EsriPolygon<N>> {
        match self {
            EsriGeometry::Polygon(ply) => Some(ply),
            _ => None,
        }
    }
}

// Completed: esriGeometryPoint | esriGeometryMultipoint | esriGeometryPolyline | esriGeometryPolygon |
// TODO: esriGeometryEnvelope.

#[allow(non_snake_case)]
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
#[skip_serializing_none]
pub struct EsriEnvelope {
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64,
    zmin: Option<f64>,
    zmax: Option<f64>,
    mmin: Option<f64>,
    mmax: Option<f64>,
    spatialReference: Option<SpatialReference>,
}
