//! Converts Esri JSON geometries into engine geometries.
//!
//! Coordinates hold x and y, then Z when `hasZ` and M when `hasM`; without either flag the
//! coordinate width decides. Polygon rings drop a closing vertex equal to their first, since the
//! engine keeps rings implicitly closed.

use crate::{
    enginex::{
        Envelope, Envelope2D, Geometry, Interval, MultiPath, MultiPoint, Point, Polygon, Polyline,
        Vertex,
    },
    geometry::{EsriCoord, EsriGeometry, EsriLineString},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FromEsriError {
    /// `hasZ` and `hasM` call for a different number of ordinates than each coordinate holds.
    Dimensions { expected: usize, found: usize },
    /// A path offset does not fit the engine's 32-bit integers.
    TooLarge,
}

impl std::fmt::Display for FromEsriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromEsriError::Dimensions { expected, found } => write!(
                f,
                "hasZ and hasM need {expected} ordinates per coordinate, found {found}"
            ),
            FromEsriError::TooLarge => write!(f, "geometry exceeds 32-bit path offsets"),
        }
    }
}

impl std::error::Error for FromEsriError {}

/// Where Z and M sit in an Esri coordinate: after x and y, Z first when both are present.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Layout {
    pub(super) z: Option<usize>,
    pub(super) m: Option<usize>,
}

impl Layout {
    /// The layout `hasZ` and `hasM` describe, or the one a coordinate of width `N` implies.
    pub(super) fn new<const N: usize>(
        has_z: Option<bool>,
        has_m: Option<bool>,
    ) -> Result<Self, FromEsriError> {
        let (z, m) = match (has_z, has_m) {
            (None, None) => (N >= 3, N >= 4),
            (z, m) => (z == Some(true), m == Some(true)),
        };
        let expected = 2 + usize::from(z) + usize::from(m);
        if expected != N {
            return Err(FromEsriError::Dimensions { expected, found: N });
        }
        Ok(Layout {
            z: z.then_some(2),
            m: m.then_some(if z { 3 } else { 2 }),
        })
    }

    pub(super) fn vertex<const N: usize>(self, coord: &EsriCoord<N>) -> Vertex {
        let at = |i: usize| coord.0.get(i).copied();
        Vertex {
            x: at(0).unwrap_or(f64::NAN),
            y: at(1).unwrap_or(f64::NAN),
            z: self.z.and_then(at),
            m: self.m.and_then(at),
            id: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PathType {
    Polyline,
    Polygon,
}

impl MultiPath {
    /// Esri paths as engine paths; polygon rings drop a closing vertex equal to their first.
    fn from_esri<const N: usize>(
        paths: &[EsriLineString<N>],
        layout: Layout,
        path_type: PathType,
    ) -> Result<MultiPath, FromEsriError> {
        let mut vertices = Vec::new();
        let mut path_offsets = vec![0];
        for path in paths {
            let mut coords = path.0.as_slice();
            if let (PathType::Polygon, [first, .., last]) = (path_type, coords) {
                if first.0 == last.0 {
                    coords = &coords[..coords.len() - 1];
                }
            }
            vertices.extend(coords.iter().map(|c| layout.vertex(c)));
            let end = i32::try_from(vertices.len()).map_err(|_| FromEsriError::TooLarge)?;
            path_offsets.push(end);
        }
        Ok(MultiPath {
            vertices: vertices.into_iter().collect(),
            path_flags: vec![Default::default(); paths.len()],
            path_offsets,
            segments: None,
        })
    }
}

impl<const N: usize> TryFrom<&EsriGeometry<N>> for Geometry {
    type Error = FromEsriError;

    fn try_from(geometry: &EsriGeometry<N>) -> Result<Self, Self::Error> {
        Ok(match geometry {
            EsriGeometry::Point(p) => {
                let vertex = Vertex {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                    m: p.m,
                    id: None,
                };
                Geometry::Point(Point((!p.x.is_nan()).then_some(vertex)))
            }
            EsriGeometry::MultiPoint(mp) => {
                let layout = Layout::new::<N>(mp.hasZ, mp.hasM)?;
                Geometry::MultiPoint(MultiPoint {
                    vertices: mp.points.iter().map(|c| layout.vertex(c)).collect(),
                })
            }
            EsriGeometry::Polyline(pl) => {
                let layout = Layout::new::<N>(pl.hasZ, pl.hasM)?;
                Geometry::Polyline(Polyline(MultiPath::from_esri(&pl.paths, layout, PathType::Polyline)?))
            }
            EsriGeometry::Polygon(pg) => {
                let layout = Layout::new::<N>(pg.hasZ, pg.hasM)?;
                let rings = MultiPath::from_esri(&pg.rings, layout, PathType::Polygon)?;
                Geometry::Polygon(Polygon::from(rings))
            }
            EsriGeometry::Envelope(e) => {
                let interval = |min: Option<f64>, max: Option<f64>| Some(Interval { min: min?, max: max? });
                Geometry::Envelope(Envelope {
                    xy: Some(Envelope2D {
                        xmin: e.xmin,
                        ymin: e.ymin,
                        xmax: e.xmax,
                        ymax: e.ymax,
                    }),
                    z: interval(e.zmin, e.zmax),
                    m: interval(e.mmin, e.mmax),
                    id: None,
                })
            }
        })
    }
}

#[cfg(test)]
mod tests;
