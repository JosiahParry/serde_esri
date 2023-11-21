use crate::geometry::*;
use geoarrow::geo_traits::{
    CoordTrait, LineStringTrait, MultiLineStringTrait, MultiPointTrait, PointTrait, PolygonTrait,
};

impl<'a, const N: usize> CoordTrait for &'a EsriCoord<N> {
    type T = f64;

    fn x(&self) -> Self::T {
        self.0[0]
    }

    fn y(&self) -> Self::T {
        self.0[1]
    }
}

impl PointTrait for EsriPoint {
    type T = f64;

    fn x(&self) -> Self::T {
        self.x
    }
    fn y(&self) -> Self::T {
        self.y
    }
}

impl<const N: usize> PointTrait for EsriCoord<N> {
    type T = f64;

    fn x(&self) -> Self::T {
        self.0[0]
    }
    fn y(&self) -> Self::T {
        self.0[1]
    }
}

// required lifetime for multipoint trait
impl<'a, const N: usize> PointTrait for &'a EsriCoord<N> {
    type T = f64;

    fn x(&self) -> Self::T {
        self.0[0]
    }

    fn y(&self) -> Self::T {
        self.0[1]
    }
}

impl<const N: usize> MultiPointTrait for EsriMultiPoint<N> {
    type T = f64;
    type ItemType<'a> = &'a EsriCoord<N>;
    type Iter<'a> = EsriMultiPointIterator<'a, N>;

    fn points(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn num_points(&self) -> usize {
        self.points.len()
    }

    fn point(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.points.get(i)
    }
}

// Linestring compatibility for Vec<EsriCoord<N>> which is what
// is used inside of Polyline and Polygon
// Implement LineStringTrait for Vec<EsriCoord<N>>
impl<const N: usize> LineStringTrait for EsriLineString<N> {
    type T = f64;
    type ItemType<'a> = &'a EsriCoord<N>;
    type Iter<'a> = EsriLineStringIterator<'a, N>;

    fn coords(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.iter().nth(i)
    }
}

// I dont reallyyy understand the lifetimes here but hey, it compiles
impl<'a, const N: usize> LineStringTrait for &'a EsriLineString<N> {
    type T = f64;
    type ItemType<'b> = &'b EsriCoord<N> 
        where 
            Self: 'b;
    type Iter<'b> = EsriLineStringIterator<'b, N> 
        where 
            Self: 'b;

    fn coords(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.iter().nth(i)
    }
}

// Polyline implementation
impl<const N: usize> MultiLineStringTrait for EsriPolyline<N> {
    type T = f64;
    type ItemType<'a> = &'a EsriLineString<N>;
    type Iter<'a> = EsriPolylineIterator<'a, N>;

    fn lines(&self) -> Self::Iter<'_> {
        EsriPolylineIterator {
            paths_iter: self.paths.iter(),
        }
    }

    fn num_lines(&self) -> usize {
        self.paths.len()
    }

    fn line(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.paths.get(i)
    }
}

// Polygon implementation
impl<const N: usize> PolygonTrait for EsriPolygon<N> {
    type T = f64;
    type ItemType<'a> = &'a EsriLineString<N>;
    type Iter<'a> = std::iter::Skip<std::slice::Iter<'a, EsriLineString<N>>>;
    // type Iter<'a> = EsriPolygonIterator<'a, N>;

    fn exterior(&self) -> Option<Self::ItemType<'_>> {
        self.rings.iter().nth(0)
    }

    fn interiors(&self) -> Self::Iter<'_> {
        self.rings.iter().skip(1)
    }

    fn num_interiors(&self) -> usize {
        let n = self.rings.len();
        n - 1
    }

    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.rings.iter().nth(i - 1)
    }
}
