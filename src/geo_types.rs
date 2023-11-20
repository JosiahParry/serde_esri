//! Optional feature for converting Esri JSON objects into `geo-types` geometries.
//!
//! This feature enables conversion between `EsriGeometry` objects and geo_types obejcts.
//! Since geo_types presently only supports 2 dimensional coordinatees,
//! Z and M dimensions are dropped if present.
//!
//! Provides conversions for:
//!
//! - `EsriCoord` -> `Coord`
//! - `EsriPoint` -> `Point`
//! - `EsriMultiPoint` -> `MultiPoint`
//! - `EsriPolyline` -> `MultiLineString`
//! - `EsriPolygon` -> `Polygon`
use crate::geometry::*;
use geo_types::{Coord, LineString, MultiLineString, MultiPoint, Point, Polygon};

/// Note that only x and y dimensions are captured
impl<const N: usize> From<EsriCoord<N>> for Coord {
    fn from(value: EsriCoord<N>) -> Self {
        Coord {
            x: value.0[0],
            y: value.0[1],
        }
    }
}

impl From<EsriPoint> for Point {
    fn from(value: EsriPoint) -> Self {
        Point::new(value.x, value.y)
    }
}

impl<const N: usize> From<EsriMultiPoint<N>> for MultiPoint {
    fn from(value: EsriMultiPoint<N>) -> Self {
        let pnts = value
            .points
            .into_iter()
            .map(|xi| Point::from(Coord::from(xi)))
            .collect::<Vec<Point>>();

        MultiPoint::new(pnts)
    }
}

impl<const N: usize> From<EsriPolyline<N>> for MultiLineString {
    fn from(value: EsriPolyline<N>) -> Self {
        let lns = value
            .paths
            .into_iter()
            .map(|mli| {
                let li_coords = mli
                    .into_iter()
                    .map(|ci| Coord::from(ci))
                    .collect::<Vec<Coord>>();

                LineString::new(li_coords)
            })
            .collect::<Vec<LineString>>();

        MultiLineString::new(lns)
    }
}

impl<const N: usize> From<EsriPolygon<N>> for Polygon {
    fn from(value: EsriPolygon<N>) -> Self {
        let lns = value
            .rings
            .into_iter()
            .map(|mli| {
                let li_coords = mli
                    .into_iter()
                    .map(|ci| Coord::from(ci))
                    .collect::<Vec<Coord>>();

                LineString::new(li_coords)
            })
            .collect::<Vec<LineString>>();

        let mut lns_iter = lns.into_iter();
        let ext = lns_iter.next().unwrap();
        let ints = lns_iter.collect::<Vec<_>>();
        Polygon::new(ext, ints)
    }
}
