//! Builds engine geometries from any [`geo_traits`] geometry, the way the engine imports OGC
//! geometries (`OperatorImportFromWkbLocal`).
//!
//! Line strings and multilinestrings become polylines. Polygons and multipolygons become one
//! polygon whose OGC polygons start at each exterior ring; rings drop a closing vertex equal to
//! their first and are turned clockwise when exterior and counterclockwise when holes.

use crate::enginex::{
    flags::{PathFlag, PathFlags},
    geometry::{
        Envelope, Envelope2D, Geometry, Interval, Line, MultiPath, MultiPoint, Point, Polygon,
        Polyline,
    },
    vertex::{Vertex, VertexAttributes},
};
use geo_traits::{
    CoordTrait, Dimensions, GeometryTrait, GeometryType, LineStringTrait, LineTrait,
    MultiLineStringTrait, MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
    TriangleTrait,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FromGeoTraitsError {
    GeometryCollection,
    TooLarge,
}

impl std::fmt::Display for FromGeoTraitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromGeoTraitsError::GeometryCollection => {
                write!(f, "the engine has no geometry collection")
            }
            FromGeoTraitsError::TooLarge => write!(f, "geometry exceeds 32-bit path offsets"),
        }
    }
}

impl std::error::Error for FromGeoTraitsError {}

impl Vertex {
    /// A vertex with the Z and M ordinates the coordinate's dimensions carry.
    fn from_coord(coord: &impl CoordTrait<T = f64>) -> Self {
        let (z, m) = match coord.dim() {
            Dimensions::Xyz => (coord.nth(2), None),
            Dimensions::Xym => (None, coord.nth(2)),
            Dimensions::Xyzm => (coord.nth(2), coord.nth(3)),
            Dimensions::Xy | Dimensions::Unknown(_) => (None, None),
        };
        Vertex {
            x: coord.x(),
            y: coord.y(),
            z,
            m,
            id: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RingRole {
    Exterior,
    Hole,
}

/// Paths gathered one at a time into a [`MultiPath`].
#[derive(Default)]
struct Paths {
    vertices: Vec<Vertex>,
    path_offsets: Vec<i32>,
    path_flags: Vec<PathFlags>,
    roles: Vec<RingRole>,
}

impl Paths {
    fn push_path(&mut self, vertices: impl Iterator<Item = Vertex>, flags: PathFlags) -> Result<(), FromGeoTraitsError> {
        if self.path_offsets.is_empty() {
            self.path_offsets.push(0);
        }
        self.vertices.extend(vertices);
        let end = i32::try_from(self.vertices.len()).map_err(|_| FromGeoTraitsError::TooLarge)?;
        self.path_offsets.push(end);
        self.path_flags.push(flags);
        Ok(())
    }

    /// Adds a ring without the closing vertex that repeats its first.
    fn push_ring(&mut self, ring: &impl LineStringTrait<T = f64>, role: RingRole) -> Result<(), FromGeoTraitsError> {
        let mut vertices = ring.coords().map(|c| Vertex::from_coord(&c)).collect::<Vec<_>>();
        let same = |a: f64, b: f64| a == b || (a.is_nan() && b.is_nan());
        let same_option = |a: Option<f64>, b: Option<f64>| match (a, b) {
            (Some(a), Some(b)) => same(a, b),
            (a, b) => a.is_none() && b.is_none(),
        };
        if let [first, .., last] = vertices.as_slice() {
            if same(first.x, last.x)
                && same(first.y, last.y)
                && same_option(first.z, last.z)
                && same_option(first.m, last.m)
            {
                vertices.pop();
            }
        }
        let mut flags = PathFlags::from(PathFlag::Closed);
        if role == RingRole::Exterior {
            flags.insert(PathFlag::OgcStartPolygon);
        }
        self.roles.push(role);
        self.push_path(vertices.into_iter(), flags)
    }

    fn push_polygon(&mut self, polygon: &impl PolygonTrait<T = f64>) -> Result<(), FromGeoTraitsError> {
        if let Some(exterior) = polygon.exterior() {
            self.push_ring(&exterior, RingRole::Exterior)?;
        }
        for hole in polygon.interiors() {
            self.push_ring(&hole, RingRole::Hole)?;
        }
        Ok(())
    }

    fn into_multi_path(self) -> MultiPath {
        MultiPath {
            vertices: self.vertices.into_iter().collect::<VertexAttributes>(),
            path_offsets: if self.path_offsets.is_empty() {
                vec![0]
            } else {
                self.path_offsets
            },
            path_flags: self.path_flags,
            segments: None,
        }
    }

    /// Rings turned clockwise when exterior and counterclockwise when holes, as the engine does.
    fn into_polygon(self) -> Polygon {
        let roles = self.roles.clone();
        let mut rings = self.into_multi_path();
        for (i, role) in roles.into_iter().enumerate() {
            let area = rings.ring_area(i).unwrap_or(0.0);
            let is_clockwise = area > 0.0;
            if (role == RingRole::Exterior && area < 0.0) || (role == RingRole::Hole && is_clockwise) {
                rings.reverse_path(i);
            }
        }
        Polygon {
            rings,
            fill_rule: Default::default(),
        }
    }
}

impl Geometry {
    /// Builds an engine geometry from any geo-traits geometry with `f64` coordinates.
    pub fn from_geo_traits(geometry: &impl GeometryTrait<T = f64>) -> Result<Self, FromGeoTraitsError> {
        let mut paths = Paths::default();
        let open = PathFlags::default();
        Ok(match geometry.as_type() {
            GeometryType::Point(p) => Geometry::Point(Point(p.coord().map(|c| Vertex::from_coord(&c)))),
            GeometryType::Line(l) => Geometry::Line(Line {
                start: Vertex::from_coord(&l.start()),
                end: Vertex::from_coord(&l.end()),
            }),
            GeometryType::MultiPoint(mp) => Geometry::MultiPoint(MultiPoint {
                vertices: mp
                    .points()
                    .filter_map(|p| p.coord().map(|c| Vertex::from_coord(&c)))
                    .collect(),
            }),
            GeometryType::LineString(ls) => {
                if ls.num_coords() > 0 {
                    paths.push_path(ls.coords().map(|c| Vertex::from_coord(&c)), open)?;
                }
                Geometry::Polyline(Polyline(paths.into_multi_path()))
            }
            GeometryType::MultiLineString(mls) => {
                for ls in mls.line_strings().filter(|ls| ls.num_coords() > 0) {
                    paths.push_path(ls.coords().map(|c| Vertex::from_coord(&c)), open)?;
                }
                Geometry::Polyline(Polyline(paths.into_multi_path()))
            }
            GeometryType::Polygon(polygon) => {
                paths.push_polygon(polygon)?;
                Geometry::Polygon(paths.into_polygon())
            }
            GeometryType::MultiPolygon(mp) => {
                for polygon in mp.polygons() {
                    paths.push_polygon(&polygon)?;
                }
                Geometry::Polygon(paths.into_polygon())
            }
            GeometryType::Triangle(t) => {
                let ring = t.coords().map(|c| Vertex::from_coord(&c));
                let flags = PathFlags::from(PathFlag::Closed);
                paths.push_path(ring.into_iter(), flags)?;
                paths.path_flags[0].insert(PathFlag::OgcStartPolygon);
                paths.roles.push(RingRole::Exterior);
                Geometry::Polygon(paths.into_polygon())
            }
            GeometryType::Rect(r) => {
                let (min, max) = (Vertex::from_coord(&r.min()), Vertex::from_coord(&r.max()));
                let interval = |a: Option<f64>, b: Option<f64>| Some(Interval { min: a?, max: b? });
                Geometry::Envelope(Envelope {
                    xy: Some(Envelope2D {
                        xmin: min.x,
                        ymin: min.y,
                        xmax: max.x,
                        ymax: max.y,
                    }),
                    z: interval(min.z, max.z),
                    m: interval(min.m, max.m),
                    id: None,
                })
            }
            GeometryType::GeometryCollection(_) => return Err(FromGeoTraitsError::GeometryCollection),
        })
    }
}

#[cfg(test)]
mod tests;
