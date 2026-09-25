//! Converts engine geometries into [`crate::shape::Shape`] records the way the engine's shape
//! exporter (`OperatorExportToESRIShapeCursor`) writes them, using the 1998 shape types.
//!
//! Rings and closed paths regain their closing vertex, envelopes become one-ring polygons, lines
//! become one-part polylines, and `NaN` measures become "no data". Z selects the Z types and M
//! alone the M types. Vertex IDs have no place in these types and are dropped.

use crate::{
    enginex::{Envelope, Geometry, MultiPath, Point, Polygon, Polyline, Vertex, VertexAttributes},
    shape::{
        self, BoundingBox, Measures, MultiPart, MultiPartM, MultiPartZ, MultiPointM, MultiPointZ,
        PointM, PointZ, Range, Shape, ZValues,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToShapeError {
    /// Shapefiles have no curves; densify them into lines first.
    Curves,
    /// Path offsets or attribute columns do not match the vertices.
    Corrupted,
    /// A part index does not fit the format's 32-bit integers.
    TooLarge,
}

impl std::fmt::Display for ToShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToShapeError::Curves => write!(f, "shapefiles cannot hold curves"),
            ToShapeError::Corrupted => write!(f, "paths or attributes do not match the vertices"),
            ToShapeError::TooLarge => write!(f, "geometry exceeds the format's size limits"),
        }
    }
}

impl std::error::Error for ToShapeError {}

#[derive(Clone, Copy)]
enum PartsType {
    PolyLine,
    Polygon,
}

/// Vertices in shapefile order, with the envelope that bounds them.
struct Columns {
    envelope: Envelope,
    points: Vec<shape::Point>,
    z: Option<Vec<f64>>,
    m: Option<Vec<f64>>,
}

impl Columns {
    /// The vertices at `indexes`, in order.
    fn new(vertices: &VertexAttributes, indexes: &[usize]) -> Result<Self, ToShapeError> {
        let column = |values: &Option<Vec<f64>>| {
            values
                .as_ref()
                .map(|values| {
                    indexes
                        .iter()
                        .map(|&i| values.get(i).copied().ok_or(ToShapeError::Corrupted))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()
        };
        Ok(Columns {
            envelope: vertices.iter().collect(),
            points: indexes
                .iter()
                .map(|&i| {
                    let &[x, y] = vertices.xy.get(i).ok_or(ToShapeError::Corrupted)?;
                    Ok(shape::Point { x, y })
                })
                .collect::<Result<_, ToShapeError>>()?,
            z: column(&vertices.z)?,
            m: column(&vertices.m)?,
        })
    }

    fn bbox(&self) -> BoundingBox {
        let xy = self.envelope.xy;
        BoundingBox {
            xmin: xy.map_or(0.0, |e| e.xmin),
            ymin: xy.map_or(0.0, |e| e.ymin),
            xmax: xy.map_or(0.0, |e| e.xmax),
            ymax: xy.map_or(0.0, |e| e.ymax),
        }
    }

    fn z_values(&mut self) -> Option<ZValues> {
        let range = self.envelope.z.map_or(Range { min: 0.0, max: 0.0 }, |z| Range {
            min: z.min,
            max: z.max,
        });
        self.z.take().map(|values| ZValues { range, values })
    }

    /// The M section, omitted when every measure is "no data" since the section is optional.
    fn measures(&mut self) -> Option<Measures> {
        let data = |m: f64| (!m.is_nan()).then_some(m);
        let range = Range {
            min: self.envelope.m.and_then(|m| data(m.min)),
            max: self.envelope.m.and_then(|m| data(m.max)),
        };
        let values = self.m.take()?;
        if values.iter().all(|m| m.is_nan()) {
            return None;
        }
        Some(Measures {
            range,
            values: values.into_iter().map(data).collect(),
        })
    }
}

impl MultiPath {
    /// Shapefile parts, closing every path of a polygon and the closed paths of a polyline.
    fn to_shape(&self, parts_type: PartsType) -> Result<Shape, ToShapeError> {
        if self.has_nonlinear_segments() {
            return Err(ToShapeError::Curves);
        }
        let mut indexes = Vec::new();
        let mut parts = Vec::new();
        for i in 0..self.path_count() {
            let range = self.path_range(i).ok_or(ToShapeError::Corrupted)?;
            if range.is_empty() {
                continue;
            }
            parts.push(i32::try_from(indexes.len()).map_err(|_| ToShapeError::TooLarge)?);
            let first = range.start;
            indexes.extend(range);
            if matches!(parts_type, PartsType::Polygon) || self.is_closed_path(i) {
                indexes.push(first);
            }
        }

        let mut columns = Columns::new(&self.vertices, &indexes)?;
        let has_m = columns.m.is_some();
        let (bbox, z, m) = (columns.bbox(), columns.z_values(), columns.measures());
        let xy = MultiPart {
            bbox,
            parts,
            points: columns.points,
        };
        Ok(match (z, has_m, parts_type) {
            (Some(z), _, PartsType::PolyLine) => Shape::PolyLineZ(MultiPartZ { xy, z, m }),
            (Some(z), _, PartsType::Polygon) => Shape::PolygonZ(MultiPartZ { xy, z, m }),
            (None, true, PartsType::PolyLine) => Shape::PolyLineM(MultiPartM { xy, m }),
            (None, true, PartsType::Polygon) => Shape::PolygonM(MultiPartM { xy, m }),
            (None, false, PartsType::PolyLine) => Shape::PolyLine(xy),
            (None, false, PartsType::Polygon) => Shape::Polygon(xy),
        })
    }
}

impl Envelope {
    /// The engine's export of an envelope: a clockwise ring from (xmin, ymin), with Z and M
    /// alternating between their minimum and maximum.
    fn to_polygon(self) -> Polygon {
        let Some(xy) = self.xy else {
            return Polygon::from(MultiPath::default());
        };
        let corners = [
            (xy.xmin, xy.ymin),
            (xy.xmin, xy.ymax),
            (xy.xmax, xy.ymax),
            (xy.xmax, xy.ymin),
        ];
        let vertices: VertexAttributes = corners
            .into_iter()
            .enumerate()
            .map(|(i, (x, y))| {
                let pick = |interval: crate::enginex::Interval<f64>| {
                    if i % 2 == 0 {
                        interval.min
                    } else {
                        interval.max
                    }
                };
                Vertex {
                    x,
                    y,
                    z: self.z.map(pick),
                    m: self.m.map(pick),
                    id: None,
                }
            })
            .collect();
        Polygon::from(MultiPath {
            vertices,
            path_offsets: vec![0, 4],
            path_flags: vec![crate::enginex::PathFlag::Closed.into()],
            segments: None,
        })
    }
}

impl TryFrom<&Geometry> for Shape {
    type Error = ToShapeError;

    fn try_from(geometry: &Geometry) -> Result<Self, Self::Error> {
        let data = |m: f64| (!m.is_nan()).then_some(m);
        match geometry {
            Geometry::Point(Point(None)) => Ok(Shape::Point(shape::Point {
                x: f64::NAN,
                y: f64::NAN,
            })),
            Geometry::Point(Point(Some(v))) => Ok(match (v.z, v.m) {
                (Some(z), m) => Shape::PointZ(PointZ {
                    x: v.x,
                    y: v.y,
                    z,
                    m: m.and_then(data),
                }),
                (None, Some(m)) => Shape::PointM(PointM {
                    x: v.x,
                    y: v.y,
                    m: data(m),
                }),
                (None, None) => Shape::Point(shape::Point { x: v.x, y: v.y }),
            }),
            Geometry::MultiPoint(mp) => {
                let indexes: Vec<usize> = (0..mp.vertices.len()).collect();
                let mut columns = Columns::new(&mp.vertices, &indexes)?;
                let has_m = columns.m.is_some();
                let (bbox, z, m) = (columns.bbox(), columns.z_values(), columns.measures());
                let xy = shape::MultiPoint {
                    bbox,
                    points: columns.points,
                };
                Ok(match (z, has_m) {
                    (Some(z), _) => Shape::MultiPointZ(MultiPointZ { xy, z, m }),
                    (None, true) => Shape::MultiPointM(MultiPointM { xy, m }),
                    (None, false) => Shape::MultiPoint(xy),
                })
            }
            Geometry::Line(line) => {
                let path = MultiPath {
                    vertices: [line.start, line.end].into_iter().collect(),
                    path_offsets: vec![0, 2],
                    path_flags: vec![Default::default()],
                    segments: None,
                };
                path.to_shape(PartsType::PolyLine)
            }
            Geometry::Polyline(Polyline(path)) => path.to_shape(PartsType::PolyLine),
            Geometry::Polygon(polygon) => polygon.rings.to_shape(PartsType::Polygon),
            Geometry::Envelope(envelope) => envelope.to_polygon().rings.to_shape(PartsType::Polygon),
        }
    }
}

#[cfg(test)]
mod tests;
