
use geoarrow::geo_traits::{CoordTrait, PointTrait, MultiPointTrait};
use geo_types::{CoordNum, CoordFloat};
use crate::geometry::*;


impl PointTrait for EsriPoint {
    type T = f64;

    fn x(&self) -> Self::T {
        self.x 
    }
    fn y(&self) -> Self::T {
        self.y 
    }
}

impl<const N: usize> MultiPointTrait for EsriMultiPoint<N> {
    type T = f64;

    fn points(&self) -> Self::Iter<'_> {
        self.points.iter()
    }

    fn num_points(&self) -> usize {
        self.points.len()
    }

    fn point(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.points[i]
    }
}