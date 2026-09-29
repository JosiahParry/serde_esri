//! Points, lines, envelopes, multipoints, and multipaths.

use crate::enginex::{
    flags::{FillRule, GeometryType, PathFlag, PathFlags, SegmentFlags, SegmentType},
    vertex::{Vertex, VertexAttributes},
};

/// A point. `None` represents an empty point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point(pub Option<Vertex>);

/// A standalone straight line segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub start: Vertex,
    pub end: Vertex,
}

/// A closed range of attribute values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval<T> {
    pub min: T,
    pub max: T,
}

/// Two dimensional bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Envelope2D {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

/// An envelope with a range for every vertex attribute. `xy` is `None` when empty.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Envelope {
    pub xy: Option<Envelope2D>,
    pub z: Option<Interval<f64>>,
    pub m: Option<Interval<f64>>,
    pub id: Option<Interval<i32>>,
}

/// A collection of unconnected vertices.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiPoint {
    pub vertices: VertexAttributes,
}

/// Segment data of a [`MultiPath`], indexed by the vertex each segment starts at.
/// `param_index` is an offset into `params`, or `-1` when the segment has none.
///
/// Parameter layouts: `Bezier` is `[cp1_x, cp1_y, cp1_z, cp2_x, cp2_y, cp2_z]`;
/// `EllipticArc` is 8 values whose layout the engine does not define.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Segments {
    pub flags: Vec<SegmentFlags>,
    pub param_index: Vec<i32>,
    pub params: Vec<f64>,
}

impl Segments {
    /// Parameters of the segment starting at `vertex_index`, sized by its type.
    pub fn params(&self, vertex_index: usize) -> Option<&[f64]> {
        let segment_type = self.flags.get(vertex_index)?.segment_type()?;
        let start = usize::try_from(*self.param_index.get(vertex_index)?).ok()?;
        self.params.get(start..start + segment_type.param_count())
    }

    /// Control points of the Bézier segment starting at `vertex_index`.
    pub fn bezier_control_points(&self, vertex_index: usize) -> Option<[[f64; 3]; 2]> {
        if self.flags.get(vertex_index)?.segment_type()? != SegmentType::Bezier {
            return None;
        }
        match *self.params(vertex_index)? {
            [x1, y1, z1, x2, y2, z2] => Some([[x1, y1, z1], [x2, y2, z2]]),
            _ => None,
        }
    }
}

/// A vertex buffer partitioned into paths, shared by [`Polyline`] and [`Polygon`].
/// `path_offsets` has `path_count + 1` entries; `segments` is `None` when every segment is straight.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiPath {
    pub vertices: VertexAttributes,
    pub path_offsets: Vec<i32>,
    pub path_flags: Vec<PathFlags>,
    pub segments: Option<Segments>,
}

impl Default for MultiPath {
    fn default() -> Self {
        Self {
            vertices: VertexAttributes::default(),
            path_offsets: vec![0],
            path_flags: Vec::new(),
            segments: None,
        }
    }
}

impl MultiPath {
    pub fn path_count(&self) -> usize {
        self.path_offsets.len().saturating_sub(1)
    }

    pub fn path_range(&self, path_index: usize) -> Option<std::ops::Range<usize>> {
        let start = usize::try_from(*self.path_offsets.get(path_index)?).ok()?;
        let end = usize::try_from(*self.path_offsets.get(path_index + 1)?).ok()?;
        Some(start..end)
    }

    pub fn is_closed_path(&self, path_index: usize) -> bool {
        self.path_flags
            .get(path_index)
            .is_some_and(|f| f.has(PathFlag::Closed))
    }

    pub fn has_nonlinear_segments(&self) -> bool {
        self.segments.as_ref().is_some_and(|s| {
            s.flags
                .iter()
                .any(|f| f.segment_type().is_some_and(SegmentType::is_curve))
        })
    }

    /// Type of the segment starting at `vertex_index`.
    pub fn segment_type(&self, vertex_index: usize) -> Option<SegmentType> {
        match &self.segments {
            Some(s) => s.flags.get(vertex_index)?.segment_type(),
            None => (vertex_index < self.vertices.len()).then_some(SegmentType::Line),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polyline(pub MultiPath);

/// A polygon whose paths are closed rings; exterior rings are clockwise.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polygon {
    pub rings: MultiPath,
    pub fill_rule: FillRule,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Geometry {
    Point(Point),
    Line(Line),
    Envelope(Envelope),
    MultiPoint(MultiPoint),
    Polyline(Polyline),
    Polygon(Polygon),
}

impl Geometry {
    pub fn geometry_type(&self) -> GeometryType {
        match self {
            Geometry::Point(_) => GeometryType::Point,
            Geometry::Line(_) => GeometryType::Line,
            Geometry::Envelope(_) => GeometryType::Envelope,
            Geometry::MultiPoint(_) => GeometryType::MultiPoint,
            Geometry::Polyline(_) => GeometryType::Polyline,
            Geometry::Polygon(_) => GeometryType::Polygon,
        }
    }
}
