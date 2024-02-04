use crate::geometry::*;
use geoarrow::geo_traits::{
    CoordTrait, LineStringTrait, 
    LineStringIterator, MultiLineStringIterator, MultiLineStringTrait, MultiPointIterator, MultiPointTrait, PointTrait, PolygonInteriorIterator, PolygonTrait
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

    fn points(&self) -> MultiPointIterator<'_, Self::T, Self::ItemType<'_>, Self> {
        MultiPointIterator::new(self, 0, self.num_points())
    }

    fn num_points(&self) -> usize {
        self.points.len()
    }

    fn point(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.points.get(i)
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.points.get_unchecked(i)
    }
}

// // Linestring compatibility for Vec<EsriCoord<N>> which is what
// // is used inside of Polyline and Polygon
// // Implement LineStringTrait for Vec<EsriCoord<N>>
impl<const N: usize> LineStringTrait for EsriLineString<N> {
    type T = f64;
    type ItemType<'a> = &'a EsriCoord<N>;

    fn coords(&self) -> LineStringIterator<'_, Self::T, Self::ItemType<'_>, Self> {
        LineStringIterator::new(self, 0, self.num_coords())
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        if i >= self.num_coords() {
            None
        } else {
            unsafe { Some(self.coord_unchecked(i)) }
        }
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.iter().nth(i).unwrap()
    }
}

impl<const N: usize> LineStringTrait for &EsriLineString<N> {
    type T = f64;
    type ItemType<'a> = &'a EsriCoord<N> where Self: 'a;

    fn coords(&self) -> LineStringIterator<'_, Self::T, Self::ItemType<'_>, Self> {
        LineStringIterator::new(self, 0, self.num_coords())
    }

    fn num_coords(&self) -> usize {
        (*self).0.len()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        if i >= self.num_coords() {
            None
        } else {
            unsafe { Some(self.coord_unchecked(i)) }
        }
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        (*self).iter().nth(i).unwrap()
    }
}


// Polyline implementation
impl<const N: usize> MultiLineStringTrait for EsriPolyline<N> {
    type T = f64;
    type ItemType<'a> = &'a EsriLineString<N>;

    fn lines(&self) -> MultiLineStringIterator<'_, Self::T, Self::ItemType<'_>, Self> {
        MultiLineStringIterator::new(self, 0, self.num_lines())
    }

    fn num_lines(&self) -> usize {
        self.paths.len()
    }

    fn line(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.paths.get(i)
    }

    unsafe fn line_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.paths.get_unchecked(i)
    }
    
}


// // Polygon implementation
impl<const N: usize> PolygonTrait for EsriPolygon<N> {
    type T = f64;
    type ItemType<'a> = &'a EsriLineString<N>;


    fn exterior(&self) -> Option<Self::ItemType<'_>> {
        self.rings.iter().nth(0)
    }


    fn num_interiors(&self) -> usize {
        let n = self.rings.len();
        n - 1
    }

    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.rings.get(i + 1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.rings.get_unchecked(i + 1)
    }

    fn interiors(&self) -> PolygonInteriorIterator<'_, Self::T, Self::ItemType<'_>, Self> {
        PolygonInteriorIterator::new(self, 0, self.num_interiors())
    }
}
