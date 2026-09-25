//! Reads Esri shape buffers, the engine's native binary format
//! (`OperatorImportFromESRIShape`), into [`Geometry`].
//!
//! ```ignore
//! let geometry: Option<Geometry> = EsriShape(bytes).try_into()?;
//! ```
//!
//! Buffers are little endian and a null shape reads as `None`. Z and M values
//! below `-1e38` are read as `NaN`. Polygon rings drop their closing vertex
//! when it equals the first vertex, and keep it otherwise.

use crate::enginex::{
    FillRule, Geometry, MultiPath, MultiPoint, PathFlag, PathFlags, Point, Polygon, Polyline,
    Vertex, VertexAttributes,
};

const HAS_ZS: u32 = 0x8000_0000;
const HAS_MS: u32 = 0x4000_0000;
const HAS_CURVES: u32 = 0x2000_0000;
const HAS_IDS: u32 = 0x1000_0000;
const MODIFIER_MASK: u32 = 0xFF00_0000;
const BASIC_TYPE_MASK: u32 = 0x0000_00FF;

/// The bytes of one Esri shape buffer.
#[derive(Clone, Copy, Debug)]
pub struct EsriShape<'a>(pub &'a [u8]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeError {
    /// The buffer ended before the geometry was fully read.
    UnexpectedEof,
    /// The shape type is undefined, or is a multipatch, which the engine does not import.
    InvalidShapeType(u32),
    /// Part or point counts are inconsistent.
    Corrupted,
    /// The engine does not import curves from shape buffers, so their layout is undefined.
    UnsupportedCurves,
}

impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeError::UnexpectedEof => write!(f, "unexpected end of shape buffer"),
            ShapeError::InvalidShapeType(t) => write!(f, "invalid shape type {t:#x}"),
            ShapeError::Corrupted => write!(f, "corrupted geometry"),
            ShapeError::UnsupportedCurves => {
                write!(f, "shape buffers with curves are not supported")
            }
        }
    }
}

impl std::error::Error for ShapeError {}

/// Basic shape types, stored in the low byte of the shape type. Discriminants match `ShapeType`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShapeType {
    Null = 0,
    Point = 1,
    Polyline = 3,
    Polygon = 5,
    MultiPoint = 8,
    PointZ = 9,
    PolylineZ = 10,
    PointZM = 11,
    PolylineZM = 13,
    PolygonZM = 15,
    MultiPointZM = 18,
    PolygonZ = 19,
    MultiPointZ = 20,
    PointM = 21,
    PolylineM = 23,
    PolygonM = 25,
    MultiPointM = 28,
    MultiPatchM = 31,
    MultiPatch = 32,
    GeneralPolyline = 50,
    GeneralPolygon = 51,
    GeneralPoint = 52,
    GeneralMultiPoint = 53,
    GeneralMultiPatch = 54,
}

impl TryFrom<u32> for ShapeType {
    type Error = ShapeError;

    fn try_from(raw_type: u32) -> Result<Self, Self::Error> {
        use ShapeType::*;
        Ok(match raw_type & BASIC_TYPE_MASK {
            0 => Null,
            1 => Point,
            3 => Polyline,
            5 => Polygon,
            8 => MultiPoint,
            9 => PointZ,
            10 => PolylineZ,
            11 => PointZM,
            13 => PolylineZM,
            15 => PolygonZM,
            18 => MultiPointZM,
            19 => PolygonZ,
            20 => MultiPointZ,
            21 => PointM,
            23 => PolylineM,
            25 => PolygonM,
            28 => MultiPointM,
            31 => MultiPatchM,
            32 => MultiPatch,
            50 => GeneralPolyline,
            51 => GeneralPolygon,
            52 => GeneralPoint,
            53 => GeneralMultiPoint,
            54 => GeneralMultiPatch,
            _ => return Err(ShapeError::InvalidShapeType(raw_type)),
        })
    }
}

#[derive(Clone, Copy)]
enum PathType {
    Polyline,
    Polygon,
}

#[derive(Clone, Copy)]
enum GeneralType {
    Point,
    MultiPoint,
    MultiPath(PathType),
}

/// Which attribute sections follow the xy coordinates.
#[derive(Clone, Copy)]
struct Modifiers {
    z: bool,
    m: bool,
    id: bool,
}

impl From<u32> for Modifiers {
    fn from(bits: u32) -> Self {
        Self {
            z: bits & HAS_ZS != 0,
            m: bits & HAS_MS != 0,
            id: bits & HAS_IDS != 0,
        }
    }
}

/// Empty columns for each attribute the modifiers declare.
impl From<Modifiers> for VertexAttributes {
    fn from(modifiers: Modifiers) -> Self {
        Self {
            xy: Vec::new(),
            z: modifiers.z.then(Vec::new),
            m: modifiers.m.then(Vec::new),
            id: modifiers.id.then(Vec::new),
        }
    }
}

/// Vertices to keep when reading columns, or `None` to keep all of them.
struct Retained<'a>(Option<&'a [bool]>);

impl Retained<'_> {
    fn filter<T>(&self, values: Vec<T>) -> Vec<T> {
        match self.0 {
            Some(keep) => values
                .into_iter()
                .zip(keep)
                .filter_map(|(v, &k)| k.then_some(v))
                .collect(),
            None => values,
        }
    }
}

impl TryFrom<EsriShape<'_>> for Option<Geometry> {
    type Error = ShapeError;

    fn try_from(shape: EsriShape<'_>) -> Result<Self, Self::Error> {
        let mut r = Reader(shape.0);
        let raw_type = r.i32()? as u32;

        let (general, bits) = match ShapeType::try_from(raw_type)? {
            ShapeType::Null => return Ok(None),
            ShapeType::Point => (GeneralType::Point, 0),
            ShapeType::PointZ => (GeneralType::Point, HAS_ZS),
            ShapeType::PointM => (GeneralType::Point, HAS_MS),
            ShapeType::PointZM => (GeneralType::Point, HAS_ZS | HAS_MS),
            ShapeType::GeneralPoint => (GeneralType::Point, raw_type & MODIFIER_MASK),
            ShapeType::MultiPoint => (GeneralType::MultiPoint, 0),
            ShapeType::MultiPointZ => (GeneralType::MultiPoint, HAS_ZS),
            ShapeType::MultiPointM => (GeneralType::MultiPoint, HAS_MS),
            ShapeType::MultiPointZM => (GeneralType::MultiPoint, HAS_ZS | HAS_MS),
            ShapeType::GeneralMultiPoint => (GeneralType::MultiPoint, raw_type & MODIFIER_MASK),
            ShapeType::Polyline => (GeneralType::MultiPath(PathType::Polyline), 0),
            ShapeType::PolylineZ => (GeneralType::MultiPath(PathType::Polyline), HAS_ZS),
            ShapeType::PolylineM => (GeneralType::MultiPath(PathType::Polyline), HAS_MS),
            ShapeType::PolylineZM => (GeneralType::MultiPath(PathType::Polyline), HAS_ZS | HAS_MS),
            ShapeType::GeneralPolyline => (
                GeneralType::MultiPath(PathType::Polyline),
                raw_type & MODIFIER_MASK,
            ),
            ShapeType::Polygon => (GeneralType::MultiPath(PathType::Polygon), 0),
            ShapeType::PolygonZ => (GeneralType::MultiPath(PathType::Polygon), HAS_ZS),
            ShapeType::PolygonM => (GeneralType::MultiPath(PathType::Polygon), HAS_MS),
            ShapeType::PolygonZM => (GeneralType::MultiPath(PathType::Polygon), HAS_ZS | HAS_MS),
            ShapeType::GeneralPolygon => (
                GeneralType::MultiPath(PathType::Polygon),
                raw_type & MODIFIER_MASK,
            ),
            ShapeType::MultiPatch | ShapeType::MultiPatchM | ShapeType::GeneralMultiPatch => {
                return Err(ShapeError::InvalidShapeType(raw_type))
            }
        };

        if bits & HAS_CURVES != 0 {
            return Err(ShapeError::UnsupportedCurves);
        }
        let modifiers = Modifiers::from(bits);

        let geometry = match general {
            GeneralType::Point => {
                let x = r.av_f64()?;
                let y = r.av_f64()?;
                let z = if modifiers.z { Some(r.av_f64()?) } else { None };
                let m = if modifiers.m { Some(r.av_f64()?) } else { None };
                let id = if modifiers.id { Some(r.i32()?) } else { None };
                let vertex = (!x.is_nan()).then_some(Vertex { x, y, z, m, id });
                Geometry::Point(Point(vertex))
            }
            GeneralType::MultiPoint => {
                r.skip(32)?; // bounding box
                let count = r.count()?;
                let vertices = if count == 0 {
                    modifiers.into()
                } else {
                    let xy = r.xy(count)?;
                    r.vertex_attributes(modifiers, xy, Retained(None))?
                };
                Geometry::MultiPoint(MultiPoint { vertices })
            }
            GeneralType::MultiPath(path_type) => {
                let path = r.multipath(modifiers, path_type)?;
                match path_type {
                    PathType::Polyline => Geometry::Polyline(Polyline(path)),
                    PathType::Polygon => Geometry::Polygon(Polygon {
                        rings: path,
                        fill_rule: FillRule::OddEven,
                    }),
                }
            }
        };
        Ok(Some(geometry))
    }
}

/// Little endian cursor over the unread bytes of a shape buffer.
struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], ShapeError> {
        let (bytes, rest) = self
            .0
            .split_first_chunk::<N>()
            .ok_or(ShapeError::UnexpectedEof)?;
        self.0 = rest;
        Ok(*bytes)
    }

    fn skip(&mut self, n: usize) -> Result<(), ShapeError> {
        self.0 = self.0.get(n..).ok_or(ShapeError::UnexpectedEof)?;
        Ok(())
    }

    fn i32(&mut self) -> Result<i32, ShapeError> {
        self.take().map(i32::from_le_bytes)
    }

    fn f64(&mut self) -> Result<f64, ShapeError> {
        self.take().map(f64::from_le_bytes)
    }

    /// Reads an `f64`, mapping the legacy no-data marker (below `-1e38`) to `NaN`.
    fn av_f64(&mut self) -> Result<f64, ShapeError> {
        self.f64().map(|v| if v < -1.0e38 { f64::NAN } else { v })
    }

    fn count(&mut self) -> Result<usize, ShapeError> {
        usize::try_from(self.i32()?).map_err(|_| ShapeError::Corrupted)
    }

    fn xy(&mut self, count: usize) -> Result<Vec<[f64; 2]>, ShapeError> {
        if count > self.0.len() / 16 {
            return Err(ShapeError::UnexpectedEof);
        }
        (0..count).map(|_| Ok([self.f64()?, self.f64()?])).collect()
    }

    fn multipath(
        &mut self,
        modifiers: Modifiers,
        path_type: PathType,
    ) -> Result<MultiPath, ShapeError> {
        self.skip(32)?; // bounding box
        let part_count = self.count()?;
        let point_count = self.count()?;
        if point_count == 0 {
            return Ok(MultiPath {
                vertices: modifiers.into(),
                ..Default::default()
            });
        }

        // Repeated part starts describe empty parts and are collapsed.
        let mut starts: Vec<usize> = Vec::new();
        for _ in 0..part_count {
            let start = usize::try_from(self.i32()?).map_err(|_| ShapeError::Corrupted)?;
            match starts.last() {
                Some(&prev) if start < prev => return Err(ShapeError::Corrupted),
                Some(&prev) if start == prev => {}
                _ => starts.push(start),
            }
        }
        match (starts.first(), starts.last()) {
            (Some(0), Some(&last)) if last < point_count => {}
            _ => return Err(ShapeError::Corrupted),
        }

        let xy = self.xy(point_count)?;
        let ends = starts[1..].iter().copied().chain([point_count]);

        let mut keep = vec![true; point_count];
        let mut path_offsets = vec![0];
        let mut kept = 0;
        for (start, end) in starts.iter().copied().zip(ends) {
            let mut len = end - start;
            if matches!(path_type, PathType::Polygon) && len > 1 && xy[end - 1] == xy[start] {
                keep[end - 1] = false;
                len -= 1;
            }
            kept += len;
            path_offsets.push(i32::try_from(kept).map_err(|_| ShapeError::Corrupted)?);
        }

        let (retained, flags) = match path_type {
            PathType::Polyline => (Retained(None), PathFlags::default()),
            PathType::Polygon => (Retained(Some(&keep)), PathFlag::Closed.into()),
        };

        Ok(MultiPath {
            vertices: self.vertex_attributes(modifiers, xy, retained)?,
            path_offsets,
            path_flags: vec![flags; starts.len()],
            segments: None,
        })
    }

    /// Reads the Z, M, and ID sections that follow the xy coordinates.
    fn vertex_attributes(
        &mut self,
        modifiers: Modifiers,
        xy: Vec<[f64; 2]>,
        retained: Retained,
    ) -> Result<VertexAttributes, ShapeError> {
        let count = xy.len();
        let read_f64s = |r: &mut Self| -> Result<Vec<f64>, ShapeError> {
            r.skip(16)?; // range
            (0..count).map(|_| r.av_f64()).collect()
        };

        let z = if modifiers.z {
            Some(retained.filter(read_f64s(self)?))
        } else {
            None
        };
        let m = if modifiers.m {
            Some(retained.filter(read_f64s(self)?))
        } else {
            None
        };
        let id = if modifiers.id {
            let ids = (0..count)
                .map(|_| self.i32())
                .collect::<Result<Vec<_>, _>>()?;
            Some(retained.filter(ids))
        } else {
            None
        };

        Ok(VertexAttributes {
            xy: retained.filter(xy),
            z,
            m,
            id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Buf(Vec<u8>);

    impl Buf {
        fn i32(mut self, v: i32) -> Self {
            self.0.extend(v.to_le_bytes());
            self
        }

        fn u32(self, v: u32) -> Self {
            self.i32(v as i32)
        }

        fn f64s(mut self, vs: &[f64]) -> Self {
            for v in vs {
                self.0.extend(v.to_le_bytes());
            }
            self
        }

        fn read(&self) -> Result<Option<Geometry>, ShapeError> {
            EsriShape(&self.0).try_into()
        }
    }

    #[test]
    fn null_shape() {
        assert_eq!(Buf::default().i32(0).read(), Ok(None));
    }

    #[test]
    fn point() {
        let vertex = Vertex {
            x: 1.0,
            y: 2.0,
            ..Default::default()
        };
        assert_eq!(
            Buf::default().i32(1).f64s(&[1.0, 2.0]).read(),
            Ok(Some(Geometry::Point(Point(Some(vertex)))))
        );
    }

    #[test]
    fn empty_point() {
        assert_eq!(
            Buf::default().i32(1).f64s(&[f64::NAN, f64::NAN]).read(),
            Ok(Some(Geometry::Point(Point(None))))
        );
    }

    #[test]
    fn general_point_with_zm_ids() -> Result<(), ShapeError> {
        let geometry = Buf::default()
            .u32(52 | HAS_ZS | HAS_MS | HAS_IDS)
            .f64s(&[1.0, 2.0, 3.0, -f64::MAX])
            .i32(7)
            .read()?;
        let Some(Geometry::Point(Point(Some(v)))) = geometry else {
            panic!("expected point, got {geometry:?}");
        };
        assert_eq!((v.x, v.y, v.z, v.id), (1.0, 2.0, Some(3.0), Some(7)));
        assert!(v.m.is_some_and(f64::is_nan));
        Ok(())
    }

    #[test]
    fn multipoint_z() -> Result<(), ShapeError> {
        let geometry = Buf::default()
            .i32(20)
            .f64s(&[0.0, 0.0, 1.0, 1.0])
            .i32(2)
            .f64s(&[0.0, 0.0, 1.0, 1.0])
            .f64s(&[5.0, 6.0, 5.0, 6.0])
            .read()?;
        let Some(Geometry::MultiPoint(mp)) = geometry else {
            panic!("expected multipoint, got {geometry:?}");
        };
        assert_eq!(mp.vertices.xy, vec![[0.0, 0.0], [1.0, 1.0]]);
        assert_eq!(mp.vertices.z, Some(vec![5.0, 6.0]));
        assert_eq!(mp.vertices.m, None);
        Ok(())
    }

    #[test]
    fn polygon_drops_closing_vertex_and_keeps_open_ring() -> Result<(), ShapeError> {
        // Ring 0 is closed (4 points), ring 1 is not closed (3 points).
        let geometry = Buf::default()
            .i32(25)
            .f64s(&[0.0, 0.0, 10.0, 10.0])
            .i32(2)
            .i32(7)
            .i32(0)
            .i32(4)
            .f64s(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0])
            .f64s(&[5.0, 5.0, 6.0, 5.0, 6.0, 6.0])
            .f64s(&[0.0, 6.0])
            .f64s(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
            .read()?;
        let Some(Geometry::Polygon(Polygon { rings, .. })) = geometry else {
            panic!("expected polygon, got {geometry:?}");
        };
        assert_eq!(rings.path_offsets, vec![0, 3, 6]);
        assert_eq!(
            rings.vertices.xy,
            vec![
                [0.0, 0.0],
                [0.0, 1.0],
                [1.0, 1.0],
                [5.0, 5.0],
                [6.0, 5.0],
                [6.0, 6.0]
            ]
        );
        assert_eq!(rings.vertices.m, Some(vec![0.0, 1.0, 2.0, 4.0, 5.0, 6.0]));
        assert!(rings.is_closed_path(0) && rings.is_closed_path(1));
        Ok(())
    }

    #[test]
    fn polyline_collapses_empty_parts() -> Result<(), ShapeError> {
        let geometry = Buf::default()
            .i32(3)
            .f64s(&[0.0, 0.0, 3.0, 3.0])
            .i32(3)
            .i32(4)
            .i32(0)
            .i32(2)
            .i32(2)
            .f64s(&[0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0])
            .read()?;
        let Some(Geometry::Polyline(Polyline(path))) = geometry else {
            panic!("expected polyline, got {geometry:?}");
        };
        assert_eq!(path.path_offsets, vec![0, 2, 4]);
        assert_eq!(path.path_flags, vec![PathFlags::default(); 2]);
        Ok(())
    }

    #[test]
    fn errors() {
        assert_eq!(
            Buf::default().u32(51 | HAS_CURVES).read(),
            Err(ShapeError::UnsupportedCurves)
        );
        assert_eq!(
            Buf::default().i32(32).read(),
            Err(ShapeError::InvalidShapeType(32))
        );
        assert_eq!(
            Buf::default().i32(1).f64s(&[1.0]).read(),
            Err(ShapeError::UnexpectedEof)
        );
        assert_eq!(
            Buf::default()
                .i32(3)
                .f64s(&[0.0; 4])
                .i32(2)
                .i32(2)
                .i32(1)
                .i32(0)
                .read(),
            Err(ShapeError::Corrupted)
        );
    }
}
