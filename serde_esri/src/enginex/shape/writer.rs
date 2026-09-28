//! Writes engine geometries as Esri shape buffers the way the engine's shape exporter
//! (`OperatorExportToESRIShapeCursor`) does, the inverse of reading an [`EsriShape`].
//!
//! Vertices with IDs use the general shape types with modifier bits; others use the basic types,
//! including the Z-only codes. Z and M write `NaN` as the legacy "no data" value `-f64::MAX`,
//! empty bounding boxes are `NaN`, and rings and closed paths gain their closing vertex.

use crate::enginex::{
    shape::{EsriShape, ShapeError, ShapeType, HAS_IDS, HAS_MS, HAS_ZS},
    Envelope, Geometry, MultiPath, Point, Polyline, Attribute, Vertex, VertexAttributes,
    VertexDescription,
};

/// An Esri shape buffer written from an engine geometry. The default is the null shape.
#[derive(Clone, Debug, PartialEq)]
pub struct EsriShapeBuffer(pub Vec<u8>);

impl EsriShapeBuffer {
    /// Borrows the buffer to read it back.
    pub fn as_shape(&self) -> EsriShape<'_> {
        EsriShape(&self.0)
    }
}

impl Default for EsriShapeBuffer {
    fn default() -> Self {
        EsriShapeBuffer(ShapeType::Null.code().to_le_bytes().to_vec())
    }
}

impl ShapeType {
    fn code(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Point,
    MultiPoint,
    Polyline,
    Polygon,
}

impl Kind {
    /// The basic type for these attributes, or the general type with modifier bits when there are IDs.
    fn type_code(self, description: VertexDescription) -> u32 {
        let (z, m) = (description.has(Attribute::Z), description.has(Attribute::M));
        if description.has(Attribute::Id) {
            let general = match self {
                Kind::Point => ShapeType::GeneralPoint,
                Kind::MultiPoint => ShapeType::GeneralMultiPoint,
                Kind::Polyline => ShapeType::GeneralPolyline,
                Kind::Polygon => ShapeType::GeneralPolygon,
            };
            let z_bit = if z { HAS_ZS } else { 0 };
            let m_bit = if m { HAS_MS } else { 0 };
            return general.code() | HAS_IDS | z_bit | m_bit;
        }
        let basic = match (self, z, m) {
            (Kind::Point, false, false) => ShapeType::Point,
            (Kind::Point, true, false) => ShapeType::PointZ,
            (Kind::Point, false, true) => ShapeType::PointM,
            (Kind::Point, true, true) => ShapeType::PointZM,
            (Kind::MultiPoint, false, false) => ShapeType::MultiPoint,
            (Kind::MultiPoint, true, false) => ShapeType::MultiPointZ,
            (Kind::MultiPoint, false, true) => ShapeType::MultiPointM,
            (Kind::MultiPoint, true, true) => ShapeType::MultiPointZM,
            (Kind::Polyline, false, false) => ShapeType::Polyline,
            (Kind::Polyline, true, false) => ShapeType::PolylineZ,
            (Kind::Polyline, false, true) => ShapeType::PolylineM,
            (Kind::Polyline, true, true) => ShapeType::PolylineZM,
            (Kind::Polygon, false, false) => ShapeType::Polygon,
            (Kind::Polygon, true, false) => ShapeType::PolygonZ,
            (Kind::Polygon, false, true) => ShapeType::PolygonM,
            (Kind::Polygon, true, true) => ShapeType::PolygonZM,
        };
        basic.code()
    }
}

/// Appends little endian shape buffer fields.
struct Writer(Vec<u8>);

impl Writer {
    fn u32(&mut self, v: u32) {
        self.0.extend(v.to_le_bytes());
    }

    fn i32(&mut self, v: i32) {
        self.0.extend(v.to_le_bytes());
    }

    fn f64(&mut self, v: f64) {
        self.0.extend(v.to_le_bytes());
    }

    /// Writes `NaN` as the legacy "no data" value (`Interop.translateToAVNaN`).
    fn av_f64(&mut self, v: f64) {
        self.f64(if v.is_nan() { f64::MIN } else { v });
    }

    fn count(&mut self, n: usize) -> Result<(), ShapeError> {
        self.i32(i32::try_from(n).map_err(|_| ShapeError::Corrupted)?);
        Ok(())
    }

    /// The x/y bounds of every vertex, `NaN` when there are none.
    fn bbox(&mut self, envelope: &Envelope) {
        let bounds = envelope.xy.map_or([f64::NAN; 4], |e| [e.xmin, e.ymin, e.xmax, e.ymax]);
        for v in bounds {
            self.f64(v);
        }
    }

    /// Writes x/y, then the Z, M, and ID sections, for the vertices at `indexes`.
    fn attributes(
        &mut self,
        vertices: &VertexAttributes,
        envelope: &Envelope,
        indexes: &[usize],
    ) -> Result<(), ShapeError> {
        for &i in indexes {
            let &[x, y] = vertices.xy.get(i).ok_or(ShapeError::Corrupted)?;
            self.f64(x);
            self.f64(y);
        }
        for (column, range) in [(&vertices.z, envelope.z), (&vertices.m, envelope.m)] {
            let Some(values) = column else {
                continue;
            };
            self.av_f64(range.map_or(f64::NAN, |r| r.min));
            self.av_f64(range.map_or(f64::NAN, |r| r.max));
            for &i in indexes {
                self.av_f64(*values.get(i).ok_or(ShapeError::Corrupted)?);
            }
        }
        if let Some(ids) = &vertices.id {
            for &i in indexes {
                self.i32(*ids.get(i).ok_or(ShapeError::Corrupted)?);
            }
        }
        Ok(())
    }

    /// Writes a multipath's parts, closing every polygon ring and each closed polyline path.
    fn multi_path(&mut self, path: &MultiPath, kind: Kind) -> Result<(), ShapeError> {
        if path.has_nonlinear_segments() {
            return Err(ShapeError::UnsupportedCurves);
        }
        let mut indexes = Vec::new();
        let mut starts = Vec::new();
        for i in 0..path.path_count() {
            let range = path.path_range(i).ok_or(ShapeError::Corrupted)?;
            starts.push(indexes.len());
            let first = range.start;
            let is_empty = range.is_empty();
            indexes.extend(range);
            if !is_empty && (matches!(kind, Kind::Polygon) || path.is_closed_path(i)) {
                indexes.push(first);
            }
        }

        let envelope: Envelope = path.vertices.iter().collect();
        self.u32(kind.type_code(path.vertices.description()));
        self.bbox(&envelope);
        self.count(starts.len())?;
        self.count(indexes.len())?;
        for start in starts {
            self.count(start)?;
        }
        self.attributes(&path.vertices, &envelope, &indexes)
    }
}

impl TryFrom<&Geometry> for EsriShapeBuffer {
    type Error = ShapeError;

    fn try_from(geometry: &Geometry) -> Result<Self, Self::Error> {
        let mut w = Writer(Vec::new());
        match geometry {
            Geometry::Point(Point(vertex)) => {
                let description = vertex.map_or(VertexDescription::XY, |v| v.description());
                let v = vertex.unwrap_or(Vertex {
                    x: f64::NAN,
                    y: f64::NAN,
                    ..Default::default()
                });
                w.u32(Kind::Point.type_code(description));
                w.av_f64(v.x);
                w.av_f64(v.y);
                if description.has(Attribute::Z) {
                    w.av_f64(v.z.unwrap_or(f64::NAN));
                }
                if description.has(Attribute::M) {
                    w.av_f64(v.m.unwrap_or(f64::NAN));
                }
                if description.has(Attribute::Id) {
                    w.i32(v.id.unwrap_or(0));
                }
            }
            Geometry::MultiPoint(mp) => {
                let envelope: Envelope = mp.vertices.iter().collect();
                let indexes: Vec<usize> = (0..mp.vertices.len()).collect();
                w.u32(Kind::MultiPoint.type_code(mp.vertices.description()));
                w.bbox(&envelope);
                w.count(indexes.len())?;
                w.attributes(&mp.vertices, &envelope, &indexes)?;
            }
            Geometry::Line(line) => {
                let path = MultiPath {
                    vertices: [line.start, line.end].into_iter().collect(),
                    path_offsets: vec![0, 2],
                    path_flags: vec![Default::default()],
                    segments: None,
                };
                w.multi_path(&path, Kind::Polyline)?;
            }
            Geometry::Polyline(Polyline(path)) => w.multi_path(path, Kind::Polyline)?,
            Geometry::Polygon(polygon) => w.multi_path(&polygon.rings, Kind::Polygon)?,
            Geometry::Envelope(envelope) => {
                w.multi_path(&envelope.to_polygon().rings, Kind::Polygon)?
            }
        }
        Ok(EsriShapeBuffer(w.0))
    }
}

#[cfg(test)]
mod tests;
