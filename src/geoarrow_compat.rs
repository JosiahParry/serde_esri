
use geoarrow::geo_traits::{CoordTrait, PointTrait, MultiPointTrait};
use crate::geometry::*;


impl<const N: usize> CoordTrait for EsriCoord<N> {
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