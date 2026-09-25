//! Converts [`crate::shape::Shape`] records into engine geometries, following the engine's
//! shape importer (`OperatorImportFromESRIShapeCursor`).
//!
//! Empty parts are dropped, polygon rings drop their closing vertex when it equals the first,
//! and missing measures become `NaN`, the engine's default M value.

use crate::{
    enginex::{
        Geometry, MultiPath, MultiPoint, PathFlag, PathFlags, Point, Polyline,
        Vertex, VertexAttributes,
    },
    shape::{self, Measures, MultiPart, Shape},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FromShapeError {
    /// Null shapes carry no geometry.
    NullShape,
    /// The engine has no MultiPatch geometry.
    MultiPatch,
    /// Part indexes are out of order or skip the first point, or Z or M counts disagree with the points.
    Corrupted,
}

impl std::fmt::Display for FromShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromShapeError::NullShape => write!(f, "null shapes have no geometry"),
            FromShapeError::MultiPatch => write!(f, "the engine has no MultiPatch geometry"),
            FromShapeError::Corrupted => write!(f, "shape parts or attributes are inconsistent"),
        }
    }
}

impl std::error::Error for FromShapeError {}

#[derive(Clone, Copy)]
enum PathType {
    Polyline,
    Polygon,
}

impl TryFrom<Shape> for Geometry {
    type Error = FromShapeError;

    fn try_from(shape: Shape) -> Result<Self, Self::Error> {
        let vertex = |x: f64, y: f64, z: Option<f64>, m: Option<f64>| {
            let vertex = Vertex { x, y, z, m, id: None };
            Geometry::Point(Point((!x.is_nan()).then_some(vertex)))
        };
        Ok(match shape {
            Shape::Null => return Err(FromShapeError::NullShape),
            Shape::MultiPatch(_) => return Err(FromShapeError::MultiPatch),
            Shape::Point(p) => vertex(p.x, p.y, None, None),
            Shape::PointM(p) => vertex(p.x, p.y, None, Some(p.m.unwrap_or(f64::NAN))),
            Shape::PointZ(p) => vertex(p.x, p.y, Some(p.z), Some(p.m.unwrap_or(f64::NAN))),
            Shape::MultiPoint(mp) => Geometry::MultiPoint(MultiPoint {
                vertices: VertexAttributes {
                    xy: mp.points.into_iter().map(<[f64; 2]>::from).collect(),
                    ..Default::default()
                },
            }),
            Shape::MultiPointM(mp) => {
                let n = mp.xy.points.len();
                Geometry::MultiPoint(MultiPoint {
                    vertices: VertexAttributes {
                        xy: mp.xy.points.into_iter().map(<[f64; 2]>::from).collect(),
                        m: Some(Measures::engine_values(mp.m, n)?),
                        ..Default::default()
                    },
                })
            }
            Shape::MultiPointZ(mp) => {
                let n = mp.xy.points.len();
                if mp.z.values.len() != n {
                    return Err(FromShapeError::Corrupted);
                }
                Geometry::MultiPoint(MultiPoint {
                    vertices: VertexAttributes {
                        xy: mp.xy.points.into_iter().map(<[f64; 2]>::from).collect(),
                        z: Some(mp.z.values),
                        m: Some(Measures::engine_values(mp.m, n)?),
                        id: None,
                    },
                })
            }
            Shape::PolyLine(p) => Geometry::Polyline(Polyline(p.into_multi_path(
                None,
                None,
                PathType::Polyline,
            )?)),
            Shape::Polygon(p) => Geometry::Polygon(p.into_multi_path(None, None, PathType::Polygon)?.into()),
            Shape::PolyLineM(p) => {
                let m = Measures::engine_values(p.m, p.xy.points.len())?;
                Geometry::Polyline(Polyline(p.xy.into_multi_path(
                    None,
                    Some(m),
                    PathType::Polyline,
                )?))
            }
            Shape::PolygonM(p) => {
                let m = Measures::engine_values(p.m, p.xy.points.len())?;
                Geometry::Polygon(p.xy.into_multi_path(None, Some(m), PathType::Polygon)?.into())
            }
            Shape::PolyLineZ(p) => {
                let m = Measures::engine_values(p.m, p.xy.points.len())?;
                Geometry::Polyline(Polyline(p.xy.into_multi_path(
                    Some(p.z.values),
                    Some(m),
                    PathType::Polyline,
                )?))
            }
            Shape::PolygonZ(p) => {
                let m = Measures::engine_values(p.m, p.xy.points.len())?;
                Geometry::Polygon(p.xy.into_multi_path(Some(p.z.values), Some(m), PathType::Polygon)?.into())
            }
        })
    }
}

impl From<shape::Point> for [f64; 2] {
    fn from(point: shape::Point) -> Self {
        [point.x, point.y]
    }
}


impl Measures {
    /// One M per point, with "no data" and an absent M section as `NaN`.
    fn engine_values(m: Option<Measures>, num_points: usize) -> Result<Vec<f64>, FromShapeError> {
        let Some(m) = m else {
            return Ok(vec![f64::NAN; num_points]);
        };
        if m.values.len() != num_points {
            return Err(FromShapeError::Corrupted);
        }
        Ok(m.values.into_iter().map(|v| v.unwrap_or(f64::NAN)).collect())
    }
}

impl MultiPart {
    /// Builds engine paths from parts, keeping `z` and `m` aligned with the kept vertices.
    fn into_multi_path(
        self,
        z: Option<Vec<f64>>,
        m: Option<Vec<f64>>,
        path_type: PathType,
    ) -> Result<MultiPath, FromShapeError> {
        let n = self.points.len();
        if z.as_ref().is_some_and(|z| z.len() != n) || m.as_ref().is_some_and(|m| m.len() != n) {
            return Err(FromShapeError::Corrupted);
        }

        // Repeated starts, and a start at the end of the points, describe empty parts.
        let mut starts: Vec<usize> = Vec::new();
        for &part in &self.parts {
            let start = usize::try_from(part).map_err(|_| FromShapeError::Corrupted)?;
            match starts.last() {
                Some(&prev) if start < prev => return Err(FromShapeError::Corrupted),
                Some(&prev) if start == prev => {}
                _ if start >= n => {}
                _ => starts.push(start),
            }
        }
        match starts.first() {
            None => {
                return Ok(MultiPath {
                    vertices: VertexAttributes {
                        xy: Vec::new(),
                        z: z.map(|_| Vec::new()),
                        m: m.map(|_| Vec::new()),
                        id: None,
                    },
                    ..Default::default()
                })
            }
            Some(0) => {}
            Some(_) => return Err(FromShapeError::Corrupted),
        }

        let xy: Vec<[f64; 2]> = self.points.into_iter().map(<[f64; 2]>::from).collect();
        let ends = starts[1..].iter().copied().chain([n]);
        let mut keep = vec![true; n];
        let mut path_offsets = vec![0];
        let mut kept = 0;
        for (start, end) in starts.iter().copied().zip(ends) {
            let mut len = end - start;
            if matches!(path_type, PathType::Polygon) && len > 1 && xy[end - 1] == xy[start] {
                keep[end - 1] = false;
                len -= 1;
            }
            kept += len;
            path_offsets.push(i32::try_from(kept).map_err(|_| FromShapeError::Corrupted)?);
        }

        let retain = |values: Vec<f64>| {
            values
                .into_iter()
                .zip(&keep)
                .filter_map(|(v, &k)| k.then_some(v))
                .collect::<Vec<_>>()
        };
        let flags = match path_type {
            PathType::Polyline => PathFlags::default(),
            PathType::Polygon => PathFlag::Closed.into(),
        };
        Ok(MultiPath {
            vertices: VertexAttributes {
                xy: xy
                    .into_iter()
                    .zip(&keep)
                    .filter_map(|(v, &k)| k.then_some(v))
                    .collect(),
                z: z.map(retain),
                m: m.map(retain),
                id: None,
            },
            path_offsets,
            path_flags: vec![flags; starts.len()],
            segments: None,
        })
    }
}

#[cfg(test)]
mod tests;
