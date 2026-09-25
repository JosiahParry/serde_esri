//! Reading and writing shape buffers: the contents of one main file record.

use crate::shape::{
    BoundingBox, Measures, MultiPart, MultiPartM, MultiPartZ, MultiPatch, MultiPoint, MultiPointM,
    MultiPointZ, PartType, Point, PointM, PointZ, Range, Shape, ShapeError, ShapeType, ZValues,
};

/// Reads a shape buffer: the contents of one main file record.
impl TryFrom<&[u8]> for Shape {
    type Error = ShapeError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let mut r = Reader(buf);
        let shape_type = ShapeType::try_from(r.i32()?)?;
        Ok(match shape_type {
            ShapeType::Null => Shape::Null,
            ShapeType::Point => Shape::Point(Point {
                x: r.f64()?,
                y: r.f64()?,
            }),
            ShapeType::PointM => Shape::PointM(PointM {
                x: r.f64()?,
                y: r.f64()?,
                m: r.measure()?,
            }),
            // Writers in the wild omit the measure, so a record ending after Z has none.
            ShapeType::PointZ => Shape::PointZ(PointZ {
                x: r.f64()?,
                y: r.f64()?,
                z: r.f64()?,
                m: if r.0.is_empty() { None } else { r.measure()? },
            }),
            ShapeType::MultiPoint => Shape::MultiPoint(r.multi_point()?),
            ShapeType::MultiPointM => {
                let xy = r.multi_point()?;
                let m = r.measures(xy.points.len())?;
                Shape::MultiPointM(MultiPointM { xy, m })
            }
            ShapeType::MultiPointZ => {
                let xy = r.multi_point()?;
                let z = r.z_values(xy.points.len())?;
                let m = r.measures(xy.points.len())?;
                Shape::MultiPointZ(MultiPointZ { xy, z, m })
            }
            ShapeType::PolyLine => Shape::PolyLine(r.multi_part()?),
            ShapeType::Polygon => Shape::Polygon(r.multi_part()?),
            ShapeType::PolyLineM | ShapeType::PolygonM => {
                let xy = r.multi_part()?;
                let m = r.measures(xy.points.len())?;
                let shape = MultiPartM { xy, m };
                if shape_type == ShapeType::PolyLineM {
                    Shape::PolyLineM(shape)
                } else {
                    Shape::PolygonM(shape)
                }
            }
            ShapeType::PolyLineZ | ShapeType::PolygonZ => {
                let xy = r.multi_part()?;
                let z = r.z_values(xy.points.len())?;
                let m = r.measures(xy.points.len())?;
                let shape = MultiPartZ { xy, z, m };
                if shape_type == ShapeType::PolyLineZ {
                    Shape::PolyLineZ(shape)
                } else {
                    Shape::PolygonZ(shape)
                }
            }
            ShapeType::MultiPatch => {
                let bbox = r.bbox()?;
                let num_parts = r.count()?;
                let num_points = r.count()?;
                let parts = r.parts(num_parts, num_points)?;
                let part_types = (0..num_parts)
                    .map(|_| PartType::try_from(r.i32()?))
                    .collect::<Result<_, _>>()?;
                let points = r.points(num_points)?;
                let z = r.z_values(num_points)?;
                let m = r.measures(num_points)?;
                Shape::MultiPatch(MultiPatch {
                    bbox,
                    parts,
                    part_types,
                    points,
                    z,
                    m,
                })
            }
        })
    }
}

/// Written in place of a missing measure; any value below `-1e38` is "no data".
const NO_DATA: f64 = f64::MIN;

/// Writes a shape buffer, the inverse of `Shape::try_from(&[u8])`.
/// Fails when part, part type, Z, or M counts disagree with the points.
impl TryFrom<&Shape> for Vec<u8> {
    type Error = ShapeError;

    fn try_from(shape: &Shape) -> Result<Self, Self::Error> {
        let mut w = Writer(Vec::new());
        w.i32(shape.shape_type() as i32);
        match shape {
            Shape::Null => {}
            Shape::Point(p) => {
                w.f64(p.x);
                w.f64(p.y);
            }
            Shape::PointM(p) => {
                w.f64(p.x);
                w.f64(p.y);
                w.measure(p.m);
            }
            Shape::PointZ(p) => {
                w.f64(p.x);
                w.f64(p.y);
                w.f64(p.z);
                w.measure(p.m);
            }
            Shape::MultiPoint(mp) => w.multi_point(mp)?,
            Shape::MultiPointM(mp) => {
                w.multi_point(&mp.xy)?;
                w.measures(mp.m.as_ref(), mp.xy.points.len())?;
            }
            Shape::MultiPointZ(mp) => {
                w.multi_point(&mp.xy)?;
                w.z_values(&mp.z, mp.xy.points.len())?;
                w.measures(mp.m.as_ref(), mp.xy.points.len())?;
            }
            Shape::PolyLine(p) | Shape::Polygon(p) => w.multi_part(p)?,
            Shape::PolyLineM(p) | Shape::PolygonM(p) => {
                w.multi_part(&p.xy)?;
                w.measures(p.m.as_ref(), p.xy.points.len())?;
            }
            Shape::PolyLineZ(p) | Shape::PolygonZ(p) => {
                w.multi_part(&p.xy)?;
                w.z_values(&p.z, p.xy.points.len())?;
                w.measures(p.m.as_ref(), p.xy.points.len())?;
            }
            Shape::MultiPatch(p) => {
                if p.part_types.len() != p.parts.len() {
                    return Err(ShapeError::Corrupted);
                }
                w.bbox(&p.bbox);
                w.count(p.parts.len())?;
                w.count(p.points.len())?;
                w.parts(&p.parts, p.points.len())?;
                for part_type in &p.part_types {
                    w.i32(*part_type as i32);
                }
                w.points(&p.points);
                w.z_values(&p.z, p.points.len())?;
                w.measures(p.m.as_ref(), p.points.len())?;
            }
        }
        Ok(w.0)
    }
}

/// Appends little endian shape buffer fields.
pub(super) struct Writer(pub(super) Vec<u8>);

impl Writer {
    pub(super) fn i32(&mut self, v: i32) {
        self.0.extend(v.to_le_bytes());
    }

    pub(super) fn f64(&mut self, v: f64) {
        self.0.extend(v.to_le_bytes());
    }

    pub(super) fn measure(&mut self, m: Option<f64>) {
        self.f64(m.unwrap_or(NO_DATA));
    }

    fn count(&mut self, n: usize) -> Result<(), ShapeError> {
        self.i32(i32::try_from(n).map_err(|_| ShapeError::TooLarge)?);
        Ok(())
    }

    pub(super) fn bbox(&mut self, bbox: &BoundingBox) {
        for v in [bbox.xmin, bbox.ymin, bbox.xmax, bbox.ymax] {
            self.f64(v);
        }
    }

    fn points(&mut self, points: &[Point]) {
        for p in points {
            self.f64(p.x);
            self.f64(p.y);
        }
    }

    /// Writes part start indexes, which must be ascending and within the points.
    fn parts(&mut self, parts: &[i32], num_points: usize) -> Result<(), ShapeError> {
        let in_order = parts.windows(2).all(|w| w[0] <= w[1]);
        let in_range = parts
            .iter()
            .all(|&p| usize::try_from(p).is_ok_and(|p| p <= num_points));
        if !in_order || !in_range {
            return Err(ShapeError::Corrupted);
        }
        for &part in parts {
            self.i32(part);
        }
        Ok(())
    }

    fn multi_point(&mut self, mp: &MultiPoint) -> Result<(), ShapeError> {
        self.bbox(&mp.bbox);
        self.count(mp.points.len())?;
        self.points(&mp.points);
        Ok(())
    }

    fn multi_part(&mut self, mp: &MultiPart) -> Result<(), ShapeError> {
        self.bbox(&mp.bbox);
        self.count(mp.parts.len())?;
        self.count(mp.points.len())?;
        self.parts(&mp.parts, mp.points.len())?;
        self.points(&mp.points);
        Ok(())
    }

    fn z_values(&mut self, z: &ZValues, num_points: usize) -> Result<(), ShapeError> {
        if z.values.len() != num_points {
            return Err(ShapeError::Corrupted);
        }
        self.f64(z.range.min);
        self.f64(z.range.max);
        for &v in &z.values {
            self.f64(v);
        }
        Ok(())
    }

    /// Writes the optional M section; `None` omits it.
    fn measures(&mut self, m: Option<&Measures>, num_points: usize) -> Result<(), ShapeError> {
        let Some(m) = m else {
            return Ok(());
        };
        if m.values.len() != num_points {
            return Err(ShapeError::Corrupted);
        }
        self.measure(m.range.min);
        self.measure(m.range.max);
        for &v in &m.values {
            self.measure(v);
        }
        Ok(())
    }
}

/// Cursor over the unread bytes of a shapefile.
pub(super) struct Reader<'a>(pub(super) &'a [u8]);

impl Reader<'_> {
    pub(super) fn take<const N: usize>(&mut self) -> Result<[u8; N], ShapeError> {
        let (bytes, rest) = self
            .0
            .split_first_chunk::<N>()
            .ok_or(ShapeError::UnexpectedEof)?;
        self.0 = rest;
        Ok(*bytes)
    }

    pub(super) fn i32(&mut self) -> Result<i32, ShapeError> {
        self.take().map(i32::from_le_bytes)
    }

    pub(super) fn i32_be(&mut self) -> Result<i32, ShapeError> {
        self.take().map(i32::from_be_bytes)
    }

    pub(super) fn f64(&mut self) -> Result<f64, ShapeError> {
        self.take().map(f64::from_le_bytes)
    }

    /// Reads a measure, where values smaller than `-1e38` are "no data".
    pub(super) fn measure(&mut self) -> Result<Option<f64>, ShapeError> {
        self.f64().map(|m| (m >= -1.0e38).then_some(m))
    }

    fn count(&mut self) -> Result<usize, ShapeError> {
        usize::try_from(self.i32()?).map_err(|_| ShapeError::Corrupted)
    }

    pub(super) fn bbox(&mut self) -> Result<BoundingBox, ShapeError> {
        Ok(BoundingBox {
            xmin: self.f64()?,
            ymin: self.f64()?,
            xmax: self.f64()?,
            ymax: self.f64()?,
        })
    }

    fn points(&mut self, count: usize) -> Result<Vec<Point>, ShapeError> {
        (0..count)
            .map(|_| {
                Ok(Point {
                    x: self.f64()?,
                    y: self.f64()?,
                })
            })
            .collect()
    }

    /// Reads part start indexes, which must be ascending and within the points.
    fn parts(&mut self, num_parts: usize, num_points: usize) -> Result<Vec<i32>, ShapeError> {
        let mut parts = Vec::new();
        for _ in 0..num_parts {
            let start = self.i32()?;
            let in_order = parts.last().is_none_or(|&prev| prev <= start);
            if !in_order || usize::try_from(start).map_or(true, |s| s > num_points) {
                return Err(ShapeError::Corrupted);
            }
            parts.push(start);
        }
        Ok(parts)
    }

    fn multi_point(&mut self) -> Result<MultiPoint, ShapeError> {
        let bbox = self.bbox()?;
        let num_points = self.count()?;
        Ok(MultiPoint {
            bbox,
            points: self.points(num_points)?,
        })
    }

    fn multi_part(&mut self) -> Result<MultiPart, ShapeError> {
        let bbox = self.bbox()?;
        let num_parts = self.count()?;
        let num_points = self.count()?;
        Ok(MultiPart {
            bbox,
            parts: self.parts(num_parts, num_points)?,
            points: self.points(num_points)?,
        })
    }

    fn z_values(&mut self, count: usize) -> Result<ZValues, ShapeError> {
        Ok(ZValues {
            range: Range {
                min: self.f64()?,
                max: self.f64()?,
            },
            values: (0..count).map(|_| self.f64()).collect::<Result<_, _>>()?,
        })
    }

    /// Reads an optional M section, absent when no bytes remain.
    fn measures(&mut self, count: usize) -> Result<Option<Measures>, ShapeError> {
        if self.0.is_empty() {
            return Ok(None);
        }
        Ok(Some(Measures {
            range: Range {
                min: self.measure()?,
                max: self.measure()?,
            },
            values: (0..count)
                .map(|_| self.measure())
                .collect::<Result<_, _>>()?,
        }))
    }
}
