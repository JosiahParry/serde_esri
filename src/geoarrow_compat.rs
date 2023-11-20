use crate::geometry::*;
use geoarrow::geo_traits::{CoordTrait, LineStringTrait, MultiPointTrait, PointTrait};

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
    type Iter<'a> = std::slice::Iter<'a, EsriCoord<N>>;

    fn coords(&self) -> Self::Iter<'_> {
        // self.iter()
        todo!()
    }

    fn num_coords(&self) -> usize {
        // self.len()
        todo!()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        // self.get(i)
        todo!()
    }
}
