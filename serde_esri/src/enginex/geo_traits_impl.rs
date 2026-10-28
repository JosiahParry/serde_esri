//! [`geo_traits`] implementations for engine geometries.
//!
//! A [`Polyline`] is a multilinestring and a [`Polygon`] a multipolygon grouped by its OGC
//! flags (see [`Polygon::ogc_polygons`]). Rings and closed paths repeat their first vertex,
//! since the engine leaves the closing segment implicit. Vertex IDs have no geo-traits
//! dimension and are not exposed.

use crate::enginex::{
    description::{Attribute, VertexDescription},
    geometry::{Envelope, Geometry, Line, MultiPath, MultiPoint, Point, Polygon, Polyline},
    vertex::{Vertex, VertexAttributes},
};
use geo_traits::{
    CoordTrait, Dimensions, GeometryTrait, GeometryType, LineStringTrait, LineTrait,
    MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
    UnimplementedGeometryCollection, UnimplementedLineString, UnimplementedPolygon,
    UnimplementedTriangle,
};

/// Z and M map to their dimensions; IDs have none.
impl From<VertexDescription> for Dimensions {
    fn from(description: VertexDescription) -> Self {
        match (description.has(Attribute::Z), description.has(Attribute::M)) {
            (false, false) => Dimensions::Xy,
            (true, false) => Dimensions::Xyz,
            (false, true) => Dimensions::Xym,
            (true, true) => Dimensions::Xyzm,
        }
    }
}

impl CoordTrait for Vertex {
    type T = f64;

    fn dim(&self) -> Dimensions {
        self.description().into()
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> f64 {
        match (n, self.z, self.m) {
            (0, _, _) => self.x,
            (1, _, _) => self.y,
            (2, Some(z), _) => z,
            (2, None, Some(m)) | (3, Some(_), Some(m)) => m,
            _ => panic!("vertex has no ordinate {n}"),
        }
    }
}

/// One path of a [`MultiPath`] as a line string, repeating its first vertex when closed.
#[derive(Clone, Copy, Debug)]
pub struct PathView<'a> {
    vertices: &'a VertexAttributes,
    start: usize,
    len: usize,
    is_closed: bool,
}

impl<'a> PathView<'a> {
    fn ring(path: &'a MultiPath, index: usize) -> Self {
        PathView {
            is_closed: true,
            ..Self::path(path, index)
        }
    }

    fn path(path: &'a MultiPath, index: usize) -> Self {
        let range = path.path_range(index).unwrap_or(0..0);
        PathView {
            vertices: &path.vertices,
            start: range.start,
            len: range.len(),
            is_closed: path.is_closed_path(index),
        }
    }
}

/// One OGC polygon of a [`Polygon`]: an exterior ring and the holes after it.
#[derive(Clone, Copy, Debug)]
pub struct PolygonView<'a> {
    rings: &'a MultiPath,
    start: usize,
    len: usize,
}

impl LineStringTrait for PathView<'_> {
    type CoordType<'b>
        = Vertex
    where
        Self: 'b;

    fn num_coords(&self) -> usize {
        self.len + usize::from(self.is_closed && self.len > 0)
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Vertex {
        let offset = i.checked_rem(self.len).unwrap_or(0);
        self.vertices.get(self.start + offset).unwrap_or_default()
    }
}

impl PolygonTrait for PolygonView<'_> {
    type RingType<'b>
        = PathView<'b>
    where
        Self: 'b;

    fn exterior(&self) -> Option<PathView<'_>> {
        (self.len > 0).then(|| PathView::ring(self.rings, self.start))
    }

    fn num_interiors(&self) -> usize {
        self.len.saturating_sub(1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> PathView<'_> {
        PathView::ring(self.rings, self.start + 1 + i)
    }
}

impl PointTrait for Point {
    type CoordType<'b>
        = Vertex
    where
        Self: 'b;

    fn coord(&self) -> Option<Vertex> {
        self.0
    }
}

impl MultiPointTrait for MultiPoint {
    type InnerPointType<'b>
        = Point
    where
        Self: 'b;

    fn num_points(&self) -> usize {
        self.vertices.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Point {
        Point(self.vertices.get(i))
    }
}

impl MultiLineStringTrait for Polyline {
    type InnerLineStringType<'b>
        = PathView<'b>
    where
        Self: 'b;

    fn num_line_strings(&self) -> usize {
        self.0.path_count()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> PathView<'_> {
        PathView::path(&self.0, i)
    }
}

impl MultiPolygonTrait for Polygon {
    type InnerPolygonType<'b>
        = PolygonView<'b>
    where
        Self: 'b;

    fn num_polygons(&self) -> usize {
        self.ogc_polygons().count()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> PolygonView<'_> {
        let rings = self.ogc_polygons().nth(i).unwrap_or(0..0);
        PolygonView {
            rings: &self.rings,
            start: rings.start,
            len: rings.len(),
        }
    }
}

impl LineTrait for Line {
    type CoordType<'b>
        = Vertex
    where
        Self: 'b;

    fn start(&self) -> Vertex {
        self.start
    }

    fn end(&self) -> Vertex {
        self.end
    }
}

impl Envelope {
    /// The corner whose ordinates `pick` chooses from each (min, max) pair; `NaN` x and y when empty.
    fn corner(&self, pick: fn(f64, f64) -> f64) -> Vertex {
        let (x, y) = self.xy.map_or((f64::NAN, f64::NAN), |xy| {
            (pick(xy.xmin, xy.xmax), pick(xy.ymin, xy.ymax))
        });
        Vertex {
            x,
            y,
            z: self.z.map(|z| pick(z.min, z.max)),
            m: self.m.map(|m| pick(m.min, m.max)),
            id: None,
        }
    }
}

impl RectTrait for Envelope {
    type CoordType<'b>
        = Vertex
    where
        Self: 'b;

    fn min(&self) -> Vertex {
        self.corner(|min, _| min)
    }

    fn max(&self) -> Vertex {
        self.corner(|_, max| max)
    }
}

/// Implements [`GeometryTrait`] for a type that is a single geometry kind: that kind's
/// associated type is `Self` and the rest are unimplemented.
macro_rules! geometry_trait {
    (@pick Point, Point, $unimplemented:ty) => { Self };
    (@pick LineString, LineString, $unimplemented:ty) => { Self };
    (@pick Polygon, Polygon, $unimplemented:ty) => { Self };
    (@pick MultiPoint, MultiPoint, $unimplemented:ty) => { Self };
    (@pick MultiLineString, MultiLineString, $unimplemented:ty) => { Self };
    (@pick MultiPolygon, MultiPolygon, $unimplemented:ty) => { Self };
    (@pick Rect, Rect, $unimplemented:ty) => { Self };
    (@pick Line, Line, $unimplemented:ty) => { Self };
    (@pick $variant:ident, $slot:ident, $unimplemented:ty) => { $unimplemented };
    ([$($generics:tt)*] $ty:ty, $variant:ident, |$this:ident| $dim:expr) => {
        impl<$($generics)*> ::geo_traits::GeometryTrait for $ty {
            type T = f64;
            type PointType<'b> = geometry_trait!(@pick $variant, Point, ::geo_traits::UnimplementedPoint<f64>) where Self: 'b;
            type LineStringType<'b> = geometry_trait!(@pick $variant, LineString, ::geo_traits::UnimplementedLineString<f64>) where Self: 'b;
            type PolygonType<'b> = geometry_trait!(@pick $variant, Polygon, ::geo_traits::UnimplementedPolygon<f64>) where Self: 'b;
            type MultiPointType<'b> = geometry_trait!(@pick $variant, MultiPoint, ::geo_traits::UnimplementedMultiPoint<f64>) where Self: 'b;
            type MultiLineStringType<'b> = geometry_trait!(@pick $variant, MultiLineString, ::geo_traits::UnimplementedMultiLineString<f64>) where Self: 'b;
            type MultiPolygonType<'b> = geometry_trait!(@pick $variant, MultiPolygon, ::geo_traits::UnimplementedMultiPolygon<f64>) where Self: 'b;
            type GeometryCollectionType<'b> = ::geo_traits::UnimplementedGeometryCollection<f64> where Self: 'b;
            type RectType<'b> = geometry_trait!(@pick $variant, Rect, ::geo_traits::UnimplementedRect<f64>) where Self: 'b;
            type TriangleType<'b> = ::geo_traits::UnimplementedTriangle<f64> where Self: 'b;
            type LineType<'b> = geometry_trait!(@pick $variant, Line, ::geo_traits::UnimplementedLine<f64>) where Self: 'b;

            fn dim(&self) -> ::geo_traits::Dimensions {
                let $this = self;
                $dim
            }

            fn as_type(
                &self,
            ) -> ::geo_traits::GeometryType<
                '_,
                Self::PointType<'_>,
                Self::LineStringType<'_>,
                Self::PolygonType<'_>,
                Self::MultiPointType<'_>,
                Self::MultiLineStringType<'_>,
                Self::MultiPolygonType<'_>,
                Self::GeometryCollectionType<'_>,
                Self::RectType<'_>,
                Self::TriangleType<'_>,
                Self::LineType<'_>,
            > {
                ::geo_traits::GeometryType::$variant(self)
            }
        }
    };
}


geometry_trait!([] PathView<'_>, LineString, |p| p.vertices.description().into());
geometry_trait!([] PolygonView<'_>, Polygon, |p| p.rings.vertices.description().into());
geometry_trait!([] Point, Point, |p| p.0.map_or(Dimensions::Xy, |v| v.description().into()));
geometry_trait!([] MultiPoint, MultiPoint, |mp| mp.vertices.description().into());
geometry_trait!([] Polyline, MultiLineString, |p| p.0.vertices.description().into());
geometry_trait!([] Polygon, MultiPolygon, |p| p.rings.vertices.description().into());
geometry_trait!([] Line, Line, |l| l.start.description().into());
geometry_trait!([] Envelope, Rect, |e| RectTrait::min(e).description().into());

impl GeometryTrait for Geometry {
    type T = f64;
    type PointType<'b>
        = Point
    where
        Self: 'b;
    type LineStringType<'b>
        = UnimplementedLineString<f64>
    where
        Self: 'b;
    type PolygonType<'b>
        = UnimplementedPolygon<f64>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = Polyline
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = Polygon
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = UnimplementedGeometryCollection<f64>
    where
        Self: 'b;
    type RectType<'b>
        = Envelope
    where
        Self: 'b;
    type TriangleType<'b>
        = UnimplementedTriangle<f64>
    where
        Self: 'b;
    type LineType<'b>
        = Line
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        match self {
            Geometry::Point(g) => g.dim(),
            Geometry::Line(g) => g.dim(),
            Geometry::Envelope(g) => g.dim(),
            Geometry::MultiPoint(g) => g.dim(),
            Geometry::Polyline(g) => g.dim(),
            Geometry::Polygon(g) => g.dim(),
        }
    }

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Point,
        UnimplementedLineString<f64>,
        UnimplementedPolygon<f64>,
        MultiPoint,
        Polyline,
        Polygon,
        UnimplementedGeometryCollection<f64>,
        Envelope,
        UnimplementedTriangle<f64>,
        Line,
    > {
        match self {
            Geometry::Point(g) => GeometryType::Point(g),
            Geometry::Line(g) => GeometryType::Line(g),
            Geometry::Envelope(g) => GeometryType::Rect(g),
            Geometry::MultiPoint(g) => GeometryType::MultiPoint(g),
            Geometry::Polyline(g) => GeometryType::MultiLineString(g),
            Geometry::Polygon(g) => GeometryType::MultiPolygon(g),
        }
    }
}

#[cfg(test)]
mod tests;
