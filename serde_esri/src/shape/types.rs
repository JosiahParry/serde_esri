//! Shape types, geometries, and the [`Shape`] enum.

use crate::shape::error::ShapeError;

/// Shape types. Discriminants match the specification.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeType {
    Null = 0,
    Point = 1,
    PolyLine = 3,
    Polygon = 5,
    MultiPoint = 8,
    PointZ = 11,
    PolyLineZ = 13,
    PolygonZ = 15,
    MultiPointZ = 18,
    PointM = 21,
    PolyLineM = 23,
    PolygonM = 25,
    MultiPointM = 28,
    MultiPatch = 31,
}

impl TryFrom<i32> for ShapeType {
    type Error = ShapeError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ShapeType::Null,
            1 => ShapeType::Point,
            3 => ShapeType::PolyLine,
            5 => ShapeType::Polygon,
            8 => ShapeType::MultiPoint,
            11 => ShapeType::PointZ,
            13 => ShapeType::PolyLineZ,
            15 => ShapeType::PolygonZ,
            18 => ShapeType::MultiPointZ,
            21 => ShapeType::PointM,
            23 => ShapeType::PolyLineM,
            25 => ShapeType::PolygonM,
            28 => ShapeType::MultiPointM,
            31 => ShapeType::MultiPatch,
            _ => return Err(ShapeError::InvalidShapeType(value)),
        })
    }
}

/// MultiPatch part types. Discriminants match the specification.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartType {
    TriangleStrip = 0,
    TriangleFan = 1,
    OuterRing = 2,
    InnerRing = 3,
    FirstRing = 4,
    Ring = 5,
}

impl TryFrom<i32> for PartType {
    type Error = ShapeError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => PartType::TriangleStrip,
            1 => PartType::TriangleFan,
            2 => PartType::OuterRing,
            3 => PartType::InnerRing,
            4 => PartType::FirstRing,
            5 => PartType::Ring,
            _ => return Err(ShapeError::InvalidPartType(value)),
        })
    }
}

/// Bounds stored in the order xmin, ymin, xmax, ymax.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointM {
    pub x: f64,
    pub y: f64,
    pub m: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointZ {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m: Option<f64>,
}

/// A Z range followed by one Z value per point.
#[derive(Clone, Debug, PartialEq)]
pub struct ZValues {
    pub range: Range<f64>,
    pub values: Vec<f64>,
}

/// An M range followed by one measure per point.
#[derive(Clone, Debug, PartialEq)]
pub struct Measures {
    pub range: Range<Option<f64>>,
    pub values: Vec<Option<f64>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiPoint {
    pub bbox: BoundingBox,
    pub points: Vec<Point>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiPointM {
    pub xy: MultiPoint,
    pub m: Option<Measures>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiPointZ {
    pub xy: MultiPoint,
    pub z: ZValues,
    pub m: Option<Measures>,
}

/// The PolyLine and Polygon structure. `parts` holds the index of each part's
/// first point; polygon rings repeat their first point as their last.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiPart {
    pub bbox: BoundingBox,
    pub parts: Vec<i32>,
    pub points: Vec<Point>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiPartM {
    pub xy: MultiPart,
    pub m: Option<Measures>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiPartZ {
    pub xy: MultiPart,
    pub z: ZValues,
    pub m: Option<Measures>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiPatch {
    pub bbox: BoundingBox,
    pub parts: Vec<i32>,
    pub part_types: Vec<PartType>,
    pub points: Vec<Point>,
    pub z: ZValues,
    pub m: Option<Measures>,
}

/// The contents of one main file record.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Null,
    Point(Point),
    PolyLine(MultiPart),
    Polygon(MultiPart),
    MultiPoint(MultiPoint),
    PointZ(PointZ),
    PolyLineZ(MultiPartZ),
    PolygonZ(MultiPartZ),
    MultiPointZ(MultiPointZ),
    PointM(PointM),
    PolyLineM(MultiPartM),
    PolygonM(MultiPartM),
    MultiPointM(MultiPointM),
    MultiPatch(MultiPatch),
}

impl Shape {
    pub fn shape_type(&self) -> ShapeType {
        match self {
            Shape::Null => ShapeType::Null,
            Shape::Point(_) => ShapeType::Point,
            Shape::PolyLine(_) => ShapeType::PolyLine,
            Shape::Polygon(_) => ShapeType::Polygon,
            Shape::MultiPoint(_) => ShapeType::MultiPoint,
            Shape::PointZ(_) => ShapeType::PointZ,
            Shape::PolyLineZ(_) => ShapeType::PolyLineZ,
            Shape::PolygonZ(_) => ShapeType::PolygonZ,
            Shape::MultiPointZ(_) => ShapeType::MultiPointZ,
            Shape::PointM(_) => ShapeType::PointM,
            Shape::PolyLineM(_) => ShapeType::PolyLineM,
            Shape::PolygonM(_) => ShapeType::PolygonM,
            Shape::MultiPointM(_) => ShapeType::MultiPointM,
            Shape::MultiPatch(_) => ShapeType::MultiPatch,
        }
    }

    /// True for null shapes, empty points (NaN x), and shapes without points.
    pub fn is_empty(&self) -> bool {
        match self {
            Shape::Null => true,
            Shape::Point(p) => p.x.is_nan(),
            Shape::PointM(p) => p.x.is_nan(),
            Shape::PointZ(p) => p.x.is_nan(),
            Shape::MultiPoint(mp) => mp.points.is_empty(),
            Shape::MultiPointM(mp) => mp.xy.points.is_empty(),
            Shape::MultiPointZ(mp) => mp.xy.points.is_empty(),
            Shape::PolyLine(p) | Shape::Polygon(p) => p.points.is_empty(),
            Shape::PolyLineM(p) | Shape::PolygonM(p) => p.xy.points.is_empty(),
            Shape::PolyLineZ(p) | Shape::PolygonZ(p) => p.xy.points.is_empty(),
            Shape::MultiPatch(p) => p.points.is_empty(),
        }
    }
}
