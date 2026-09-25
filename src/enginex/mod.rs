//! Pure Rust representation of the in-memory geometry model used by the
//! [Esri Geometry API for Java](https://github.com/Esri/geometry-api-java).
//!
//! Types mirror the engine's storage so its data can be used directly:
//!
//! - Vertices are stored column-wise, one buffer per vertex attribute
//!   (`MultiVertexGeometryImpl.m_vertexAttributes`). The xy buffer is
//!   interleaved, so `Vec<[f64; 2]>` has the same layout as the engine's
//!   `double[]`.
//! - Multipaths partition that vertex buffer into paths with `path_offsets`
//!   and store per-path and per-segment flags (`MultiPathImpl`).
//! - Closed paths do not repeat their first vertex; the closing segment is
//!   implicit.
//! - Non-linear segments keep their parameters in a shared `f64` buffer,
//!   addressed per vertex by an index where `-1` means no parameters.
//!
//! Supported vertex attributes are x/y, z, m, and id.
//!
//! Read geometries from Esri shape buffers with [`shape::EsriShape`].

pub mod shape;

/// A vertex attribute. Discriminants match `VertexDescription.Semantics`.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Semantics {
    Position = 0,
    Z = 1,
    M = 2,
    Id = 3,
}

impl Semantics {
    /// The bit this attribute occupies in a [`VertexDescription`].
    pub const fn bit(self) -> u16 {
        1 << self as u16
    }

    /// Value of this attribute on vertices that do not set it (`VertexDescription._defaultValues`).
    pub const fn default_value(self) -> f64 {
        match self {
            Semantics::M => f64::NAN,
            _ => 0.0,
        }
    }
}

/// Bitmask of the [`Semantics`] each vertex carries (`VertexDescription.m_semanticsBitArray`).
/// [`Semantics::Position`] is always present.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexDescription(u16);

impl VertexDescription {
    pub const XY: Self = Self(Semantics::Position.bit());

    pub const fn has(self, semantics: Semantics) -> bool {
        self.0 & semantics.bit() != 0
    }

    pub fn insert(&mut self, semantics: Semantics) {
        self.0 |= semantics.bit();
    }
}

impl Default for VertexDescription {
    fn default() -> Self {
        Self::XY
    }
}

impl FromIterator<Semantics> for VertexDescription {
    fn from_iter<I: IntoIterator<Item = Semantics>>(iter: I) -> Self {
        let mut description = Self::XY;
        for semantics in iter {
            description.insert(semantics);
        }
        description
    }
}

/// Rejects bitmasks with unsupported attributes or without [`Semantics::Position`].
impl TryFrom<u16> for VertexDescription {
    type Error = u16;

    fn try_from(bits: u16) -> Result<Self, Self::Error> {
        let supported = Semantics::Position.bit()
            | Semantics::Z.bit()
            | Semantics::M.bit()
            | Semantics::Id.bit();
        if bits & !supported != 0 || bits & Semantics::Position.bit() == 0 {
            Err(bits)
        } else {
            Ok(Self(bits))
        }
    }
}

impl From<VertexDescription> for u16 {
    fn from(description: VertexDescription) -> Self {
        description.0
    }
}

/// Geometry type codes. Discriminants match `Geometry.GeometryType`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GeometryType {
    Unknown = 0,
    Point = 1 + 0x20,
    Line = 2 + 0x40 + 0x100,
    Bezier = 3 + 0x40 + 0x100,
    EllipticArc = 4 + 0x40 + 0x100,
    Envelope = 5 + 0x40 + 0x80,
    MultiPoint = 6 + 0x20 + 0x200,
    Polyline = 7 + 0x40 + 0x200 + 0x400,
    Polygon = 8 + 0x40 + 0x80 + 0x200 + 0x400,
}

/// A single path flag. Discriminants match `PathFlags`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PathFlag {
    Closed = 1,
    HasNonlinearSegments = 2,
    OgcStartPolygon = 4,
}

/// Bitmask of [`PathFlag`]s for one path of a [`MultiPath`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PathFlags(pub u8);

impl PathFlags {
    pub const fn has(self, flag: PathFlag) -> bool {
        self.0 & flag as u8 != 0
    }

    pub fn insert(&mut self, flag: PathFlag) {
        self.0 |= flag as u8;
    }

    pub fn remove(&mut self, flag: PathFlag) {
        self.0 &= !(flag as u8);
    }
}

impl From<PathFlag> for PathFlags {
    fn from(flag: PathFlag) -> Self {
        Self(flag as u8)
    }
}

/// The shape of a segment. Discriminants match `SegmentFlags`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SegmentType {
    #[default]
    Line = 1,
    Bezier = 2,
    EllipticArc = 4,
}

impl SegmentType {
    /// Number of `f64`s the segment occupies in [`Segments::params`] (`MultiPathImpl._segmentParamSizes`).
    pub const fn param_count(self) -> usize {
        match self {
            SegmentType::Line => 0,
            SegmentType::Bezier => 6,
            SegmentType::EllipticArc => 8,
        }
    }

    pub const fn is_curve(self) -> bool {
        !matches!(self, SegmentType::Line)
    }
}

/// Per-vertex segment flags: a [`SegmentType`] in the low three bits plus modifier bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SegmentFlags(pub u8);

impl SegmentFlags {
    const TYPE_MASK: u8 = 7;
    const DENSIFIED: u8 = 8;

    /// `None` if the type bits hold a value the engine does not define.
    pub const fn segment_type(self) -> Option<SegmentType> {
        match self.0 & Self::TYPE_MASK {
            1 => Some(SegmentType::Line),
            2 => Some(SegmentType::Bezier),
            4 => Some(SegmentType::EllipticArc),
            _ => None,
        }
    }

    /// Set on segments produced by densifying a non-linear segment.
    pub const fn is_densified(self) -> bool {
        self.0 & Self::DENSIFIED != 0
    }

    pub fn mark_densified(&mut self) {
        self.0 |= Self::DENSIFIED;
    }
}

impl From<SegmentType> for SegmentFlags {
    fn from(segment_type: SegmentType) -> Self {
        Self(segment_type as u8)
    }
}

impl Default for SegmentFlags {
    fn default() -> Self {
        SegmentType::Line.into()
    }
}

/// Polygon fill rule. Discriminants match `Polygon.FillRule`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FillRule {
    #[default]
    OddEven = 0,
    Winding = 1,
}

/// A single vertex with all of its attributes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub m: Option<f64>,
    pub id: Option<i32>,
}

impl Vertex {
    pub fn description(&self) -> VertexDescription {
        [
            self.z.map(|_| Semantics::Z),
            self.m.map(|_| Semantics::M),
            self.id.map(|_| Semantics::Id),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// Column-wise vertex storage. Every present attribute column has the same length as `xy`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VertexAttributes {
    pub xy: Vec<[f64; 2]>,
    pub z: Option<Vec<f64>>,
    pub m: Option<Vec<f64>>,
    pub id: Option<Vec<i32>>,
}

impl VertexAttributes {
    pub fn len(&self) -> usize {
        self.xy.len()
    }

    pub fn is_empty(&self) -> bool {
        self.xy.is_empty()
    }

    pub fn description(&self) -> VertexDescription {
        [
            self.z.as_ref().map(|_| Semantics::Z),
            self.m.as_ref().map(|_| Semantics::M),
            self.id.as_ref().map(|_| Semantics::Id),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn get(&self, index: usize) -> Option<Vertex> {
        let [x, y] = *self.xy.get(index)?;
        Some(Vertex {
            x,
            y,
            z: self.z.as_ref().and_then(|z| z.get(index).copied()),
            m: self.m.as_ref().and_then(|m| m.get(index).copied()),
            id: self.id.as_ref().and_then(|id| id.get(index).copied()),
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = Vertex> + '_ {
        (0..self.len()).filter_map(|i| self.get(i))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_description_bits_match_java() {
        let d: VertexDescription = [Semantics::Z, Semantics::Id].into_iter().collect();
        assert_eq!(u16::from(d), 0b1011);
        assert!(d.has(Semantics::Position));
        assert!(!d.has(Semantics::M));
        assert_eq!(VertexDescription::try_from(0b1011), Ok(d));
        assert_eq!(VertexDescription::try_from(0b1_0001), Err(0b1_0001));
        assert_eq!(VertexDescription::try_from(0b0010), Err(0b0010));
    }

    #[test]
    fn segment_flags_decode() {
        let mut f = SegmentFlags::from(SegmentType::Bezier);
        assert_eq!(f.segment_type(), Some(SegmentType::Bezier));
        f.mark_densified();
        assert_eq!(f.0, 10);
        assert!(f.is_densified());
        assert_eq!(f.segment_type(), Some(SegmentType::Bezier));
        assert_eq!(SegmentFlags(3).segment_type(), None);
    }

    #[test]
    fn multipath_paths_and_segments() {
        let mut closed = PathFlags::from(PathFlag::Closed);
        closed.insert(PathFlag::HasNonlinearSegments);
        let segments = Segments {
            flags: vec![
                SegmentType::Line.into(),
                SegmentType::Bezier.into(),
                SegmentType::Line.into(),
                SegmentType::Line.into(),
                SegmentType::Line.into(),
            ],
            param_index: vec![-1, 0, -1, -1, -1],
            params: vec![1.2, 0.3, 0.0, 1.1, 0.7, 0.0],
        };
        assert_eq!(segments.params(0), None);
        assert_eq!(
            segments.bezier_control_points(1),
            Some([[1.2, 0.3, 0.0], [1.1, 0.7, 0.0]])
        );

        let path = MultiPath {
            vertices: VertexAttributes {
                xy: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [5.0, 5.0], [6.0, 6.0]],
                m: Some(vec![0.0, 1.0, 2.0, 3.0, 4.0]),
                ..Default::default()
            },
            path_offsets: vec![0, 3, 5],
            path_flags: vec![closed, PathFlags::default()],
            segments: Some(segments),
        };

        assert_eq!(path.path_count(), 2);
        assert_eq!(path.path_range(1), Some(3..5));
        assert!(path.is_closed_path(0));
        assert!(!path.is_closed_path(1));
        assert!(path.has_nonlinear_segments());
        assert_eq!(path.segment_type(1), Some(SegmentType::Bezier));
        assert_eq!(u16::from(path.vertices.description()), 0b101);
        assert_eq!(path.vertices.get(4).and_then(|v| v.m), Some(4.0));
    }

    #[test]
    fn geometry_type_codes_match_java() {
        assert_eq!(GeometryType::Polygon as i32, 1736);
        assert_eq!(GeometryType::Polyline as i32, 1607);
        assert_eq!(GeometryType::Point as i32, 33);
    }
}
