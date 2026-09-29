//! [`geo_traits`] implementations for Esri JSON geometries.
//!
//! Coordinates surface as engine [`Vertex`]es with Z and M placed by `hasZ` and `hasM` (see
//! [`EsriGeometry`]). An [`EsriPolygon`] is a multipolygon: a ring with the orientation of the
//! first non-zero ring starts a polygon and the rings after it are its holes, as the engine
//! groups them. Grouping is recomputed on each access, so prefer converting to the engine for
//! repeated use.

use crate::{
    enginex::{geometry::Point, vertex::Vertex},
    geometry::{
        to_enginex::Layout, EsriCoord, EsriEnvelope, EsriGeometry, EsriLineString, EsriMultiPoint,
        EsriPoint, EsriPolygon, EsriPolyline,
    },
};
use geo_traits::{
    Dimensions, GeometryTrait, GeometryType, LineStringTrait, MultiLineStringTrait,
    MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
    UnimplementedGeometryCollection, UnimplementedLine, UnimplementedLineString,
    UnimplementedPolygon, UnimplementedTriangle,
};
use std::ops::Range;

impl From<Layout> for Dimensions {
    fn from(layout: Layout) -> Self {
        match (layout.z, layout.m) {
            (None, None) => Dimensions::Xy,
            (Some(_), None) => Dimensions::Xyz,
            (None, Some(_)) => Dimensions::Xym,
            (Some(_), Some(_)) => Dimensions::Xyzm,
        }
    }
}

/// One Esri path or ring as a line string, with coordinates as stored.
#[derive(Clone, Copy, Debug)]
pub struct EsriPathView<'a, const N: usize> {
    coords: &'a [EsriCoord<N>],
    layout: Layout,
}

/// One polygon of an [`EsriPolygon`]: an exterior ring and the holes after it.
#[derive(Clone, Copy, Debug)]
pub struct EsriPolygonView<'a, const N: usize> {
    rings: &'a [EsriLineString<N>],
    layout: Layout,
}

impl<const N: usize> EsriPolygon<N> {
    /// Ring ranges of each polygon, grouped by orientation as the engine does.
    fn polygon_ranges(&self) -> Vec<Range<usize>> {
        let xy = |c: &EsriCoord<N>| {
            let at = |i: usize| c.0.get(i).copied().unwrap_or(f64::NAN);
            (at(0), at(1))
        };
        let area = |ring: &EsriLineString<N>| {
            let Some((x0, y0)) = ring.0.first().map(xy) else {
                return 0.0;
            };
            let next = ring.0.iter().cycle().skip(1);
            ring.0
                .iter()
                .zip(next)
                .map(|(a, b)| {
                    let ((x1, y1), (x2, y2)) = (xy(a), xy(b));
                    ((x2 - x0) - (x1 - x0)) * ((y2 - y0) + (y1 - y0)) * 0.5
                })
                .sum::<f64>()
        };
        let mut starts = Vec::new();
        let mut first_sign = 0.0;
        for (i, ring) in self.rings.iter().enumerate() {
            let area = area(ring);
            if first_sign == 0.0 && area != 0.0 {
                first_sign = area.signum();
            }
            if i == 0 || first_sign == 0.0 || area * first_sign > 0.0 {
                starts.push(i);
            }
        }
        let ends = starts.iter().skip(1).copied().chain([self.rings.len()]);
        starts.iter().copied().zip(ends).map(|(s, e)| s..e).collect()
    }

    fn layout(&self) -> Layout {
        Layout::new::<N>(self.has_z, self.has_m).unwrap_or_default()
    }
}

impl<const N: usize> LineStringTrait for EsriPathView<'_, N> {
    type CoordType<'b>
        = Vertex
    where
        Self: 'b;

    fn num_coords(&self) -> usize {
        self.coords.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Vertex {
        self.coords
            .get(i)
            .map_or_else(Vertex::default, |c| self.layout.vertex(c))
    }
}

impl<const N: usize> PolygonTrait for EsriPolygonView<'_, N> {
    type RingType<'b>
        = EsriPathView<'b, N>
    where
        Self: 'b;

    fn exterior(&self) -> Option<EsriPathView<'_, N>> {
        self.rings.first().map(|ring| EsriPathView {
            coords: &ring.0,
            layout: self.layout,
        })
    }

    fn num_interiors(&self) -> usize {
        self.rings.len().saturating_sub(1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> EsriPathView<'_, N> {
        EsriPathView {
            coords: self.rings.get(i + 1).map_or(&[], |ring| &ring.0),
            layout: self.layout,
        }
    }
}

impl PointTrait for EsriPoint {
    type CoordType<'b>
        = Vertex
    where
        Self: 'b;

    fn coord(&self) -> Option<Vertex> {
        (!self.x.is_nan()).then_some(Vertex {
            x: self.x,
            y: self.y,
            z: self.z,
            m: self.m,
            id: None,
        })
    }
}

impl<const N: usize> MultiPointTrait for EsriMultiPoint<N> {
    type InnerPointType<'b>
        = Point
    where
        Self: 'b;

    fn num_points(&self) -> usize {
        self.points.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Point {
        let layout = Layout::new::<N>(self.has_z, self.has_m).unwrap_or_default();
        Point(self.points.get(i).map(|c| layout.vertex(c)))
    }
}

impl<const N: usize> MultiLineStringTrait for EsriPolyline<N> {
    type InnerLineStringType<'b>
        = EsriPathView<'b, N>
    where
        Self: 'b;

    fn num_line_strings(&self) -> usize {
        self.paths.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> EsriPathView<'_, N> {
        EsriPathView {
            coords: self.paths.get(i).map_or(&[], |path| &path.0),
            layout: Layout::new::<N>(self.has_z, self.has_m).unwrap_or_default(),
        }
    }
}

impl<const N: usize> MultiPolygonTrait for EsriPolygon<N> {
    type InnerPolygonType<'b>
        = EsriPolygonView<'b, N>
    where
        Self: 'b;

    fn num_polygons(&self) -> usize {
        self.polygon_ranges().len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> EsriPolygonView<'_, N> {
        let range = self.polygon_ranges().get(i).cloned().unwrap_or(0..0);
        EsriPolygonView {
            rings: self.rings.get(range).unwrap_or(&[]),
            layout: self.layout(),
        }
    }
}

impl RectTrait for EsriEnvelope {
    type CoordType<'b>
        = Vertex
    where
        Self: 'b;

    fn min(&self) -> Vertex {
        Vertex {
            x: self.xmin,
            y: self.ymin,
            z: self.zmin,
            m: self.mmin,
            id: None,
        }
    }

    fn max(&self) -> Vertex {
        Vertex {
            x: self.xmax,
            y: self.ymax,
            z: self.zmax,
            m: self.mmax,
            id: None,
        }
    }
}

geometry_trait!([const N: usize] EsriPathView<'_, N>, LineString, |p| p.layout.into());
geometry_trait!([const N: usize] EsriPolygonView<'_, N>, Polygon, |p| p.layout.into());
geometry_trait!([] EsriPoint, Point, |p| p.coord().map_or(Dimensions::Xy, |v| v.description().into()));
geometry_trait!([const N: usize] EsriMultiPoint<N>, MultiPoint, |mp| {
    Layout::new::<N>(mp.has_z, mp.has_m).unwrap_or_default().into()
});
geometry_trait!([const N: usize] EsriPolyline<N>, MultiLineString, |p| {
    Layout::new::<N>(p.has_z, p.has_m).unwrap_or_default().into()
});
geometry_trait!([const N: usize] EsriPolygon<N>, MultiPolygon, |p| p.layout().into());
geometry_trait!([] EsriEnvelope, Rect, |e| RectTrait::min(e).description().into());

impl<const N: usize> GeometryTrait for EsriGeometry<N> {
    type T = f64;
    type PointType<'b>
        = EsriPoint
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
        = EsriMultiPoint<N>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = EsriPolyline<N>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = EsriPolygon<N>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = UnimplementedGeometryCollection<f64>
    where
        Self: 'b;
    type RectType<'b>
        = EsriEnvelope
    where
        Self: 'b;
    type TriangleType<'b>
        = UnimplementedTriangle<f64>
    where
        Self: 'b;
    type LineType<'b>
        = UnimplementedLine<f64>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        match self {
            EsriGeometry::Point(g) => g.dim(),
            EsriGeometry::MultiPoint(g) => g.dim(),
            EsriGeometry::Polyline(g) => g.dim(),
            EsriGeometry::Polygon(g) => g.dim(),
            EsriGeometry::Envelope(g) => g.dim(),
        }
    }

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        EsriPoint,
        UnimplementedLineString<f64>,
        UnimplementedPolygon<f64>,
        EsriMultiPoint<N>,
        EsriPolyline<N>,
        EsriPolygon<N>,
        UnimplementedGeometryCollection<f64>,
        EsriEnvelope,
        UnimplementedTriangle<f64>,
        UnimplementedLine<f64>,
    > {
        match self {
            EsriGeometry::Point(g) => GeometryType::Point(g),
            EsriGeometry::MultiPoint(g) => GeometryType::MultiPoint(g),
            EsriGeometry::Polyline(g) => GeometryType::MultiLineString(g),
            EsriGeometry::Polygon(g) => GeometryType::MultiPolygon(g),
            EsriGeometry::Envelope(g) => GeometryType::Rect(g),
        }
    }
}

#[cfg(test)]
mod tests;
