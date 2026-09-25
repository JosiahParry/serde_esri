//! Geometry, path, and segment type codes and flags.

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
