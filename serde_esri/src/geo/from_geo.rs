// This module is for converting geo types into Esri geometry objects
// requires geo for cw and ccw enforcement

use crate::geometry::*;
use geo::orient::Direction;
use geo::{BooleanOps, CoordsIter, Orient};
use geo_types::{
    Coord, Geometry, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
    Rect, Triangle,
};

// macro to implement T from &T impl
macro_rules! impl_into {
    ($Source:ty, $Target:ty) => {
        impl Into<$Target> for $Source {
            fn into(self) -> $Target {
                (&self).into()
            }
        }
    };
}

// Base data

impl Into<EsriCoord<2>> for &Coord {
    fn into(self) -> EsriCoord<2> {
        EsriCoord::<2>([self.x, self.y])
    }
}
impl_into!(Coord, EsriCoord<2>);

impl Into<EsriLineString<2>> for &LineString {
    fn into(self) -> EsriLineString<2> {
        let coords: Vec<EsriCoord<2>> =
            self.coords_iter().map(Into::<EsriCoord<2>>::into).collect();
        EsriLineString::<2>(coords)
    }
}
impl_into!(LineString, EsriLineString<2>);

// Geometries

impl Into<EsriPoint> for &Point {
    fn into(self) -> EsriPoint {
        EsriPoint {
            x: self.x(),
            y: self.y(),
            z: None,
            m: None,
            spatial_reference: None,
        }
    }
}
impl_into!(Point, EsriPoint);

impl Into<EsriMultiPoint<2>> for &MultiPoint {
    fn into(self) -> EsriMultiPoint<2> {
        let coords = self.coords_iter().map(Into::<EsriCoord<2>>::into).collect();

        EsriMultiPoint {
            has_z: None,
            has_m: None,
            points: coords,
            spatial_reference: None,
        }
    }
}
impl_into!(MultiPoint, EsriMultiPoint<2>);

impl Into<EsriPolyline<2>> for &Line {
    fn into(self) -> EsriPolyline<2> {
        let coords: Vec<EsriCoord<2>> = vec![self.start.into(), self.end.into()];

        EsriPolyline {
            has_z: None,
            has_m: None,
            paths: vec![EsriLineString::<2>(coords)],
            spatial_reference: None,
        }
    }
}
impl_into!(Line, EsriPolyline<2>);

impl Into<EsriPolyline<2>> for &LineString {
    fn into(self) -> EsriPolyline<2> {
        let line_string = Into::<EsriLineString<2>>::into(self);

        EsriPolyline {
            has_z: None,
            has_m: None,
            paths: vec![line_string],
            spatial_reference: None,
        }
    }
}
impl_into!(LineString, EsriPolyline<2>);

impl Into<EsriPolyline<2>> for &MultiLineString {
    fn into(self) -> EsriPolyline<2> {
        let line_strings = self
            .iter()
            .map(Into::<EsriLineString<2>>::into)
            .collect::<Vec<EsriLineString<2>>>();

        EsriPolyline {
            has_z: None,
            has_m: None,
            paths: line_strings,
            spatial_reference: None,
        }
    }
}
impl_into!(MultiLineString, EsriPolyline<2>);

impl Into<EsriPolygon<2>> for &Polygon {
    fn into(self) -> EsriPolygon<2> {
        let rewound = self.orient(Direction::Reversed);
        let rings: Vec<EsriLineString<2>> = rewound
            .rings()
            .map(Into::<EsriLineString<2>>::into)
            .collect();

        EsriPolygon {
            has_z: None,
            has_m: None,
            rings,
            spatial_reference: None,
        }
    }
}
impl_into!(Polygon, EsriPolygon<2>);

impl Into<EsriPolygon<2>> for &MultiPolygon {
    fn into(self) -> EsriPolygon<2> {
        let rewound = self.orient(Direction::Reversed);
        let rings = rewound
            .rings()
            .map(Into::<EsriLineString<2>>::into)
            .collect();

        EsriPolygon {
            has_z: None,
            has_m: None,
            rings,
            spatial_reference: None,
        }
    }
}
impl_into!(MultiPolygon, EsriPolygon<2>);

impl Into<EsriPolygon<2>> for &Triangle {
    fn into(self) -> EsriPolygon<2> {
        self.to_polygon().into()
    }
}
impl_into!(Triangle, EsriPolygon<2>);

impl Into<EsriPolygon<2>> for &Rect {
    fn into(self) -> EsriPolygon<2> {
        self.to_polygon().into()
    }
}
impl_into!(Rect, EsriPolygon<2>);

impl TryInto<EsriGeometry<2>> for &Geometry {
    type Error = Option<()>;

    fn try_into(self) -> Result<EsriGeometry<2>, Self::Error> {
        match self {
            Geometry::Point(g) => Ok(EsriGeometry::Point(g.into())),
            Geometry::MultiPoint(g) => Ok(EsriGeometry::MultiPoint(g.into())),

            Geometry::Line(g) => Ok(EsriGeometry::Polyline(g.into())),
            Geometry::LineString(g) => Ok(EsriGeometry::Polyline(g.into())),
            Geometry::MultiLineString(g) => Ok(EsriGeometry::Polyline(g.into())),

            Geometry::Polygon(g) => Ok(EsriGeometry::Polygon(g.into())),
            Geometry::MultiPolygon(g) => Ok(EsriGeometry::Polygon(g.into())),
            Geometry::Rect(g) => Ok(EsriGeometry::Polygon(g.into())),
            Geometry::Triangle(g) => Ok(EsriGeometry::Polygon(g.into())),

            Geometry::GeometryCollection(_g) => Err(None),
        }
    }
}
impl TryInto<EsriGeometry<2>> for Geometry {
    type Error = Option<()>;
    fn try_into(self) -> Result<EsriGeometry<2>, Self::Error> {
        (&self).try_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::*;
    use geo::{coord, Orient, Polygon, Rect};

    #[test]
    fn test_polygon() {
        let poly: Polygon<f64> = Rect::new(coord! {x:0.0,y:0.0}, coord! {x:1.0,y:1.0}).to_polygon();
        let poly1 = poly.orient(geo::orient::Direction::Default);
        let poly2 = poly.orient(geo::orient::Direction::Reversed);

        // external ring is differently wound
        assert_ne!(
            poly1.exterior().coords().collect::<Vec<_>>(),
            poly2.exterior().coords().collect::<Vec<_>>()
        );
        assert_eq!(
            poly1.exterior().coords().collect::<Vec<_>>(),
            poly2.exterior().coords().rev().collect::<Vec<_>>()
        );

        let esri_poly1: EsriPolygon<2> = poly1.into();
        let esri_poly2: EsriPolygon<2> = poly2.into();

        let serial1 = serde_json::to_string(&esri_poly1).unwrap();
        let serial2 = serde_json::to_string(&esri_poly2).unwrap();

        assert_eq!(serial1, serial2);
    }
}
