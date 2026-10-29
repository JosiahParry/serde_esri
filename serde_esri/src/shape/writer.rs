//! Streaming a main file to a seekable writer, building its index.

use crate::enginex::geometry::{Envelope, Envelope2D, Interval};
use crate::shape::{
    error::{FileError, ShapeError},
    file::FileHeader,
    index::{IndexRecord, ShapeIndex},
    types::{Measures, Shape, ShapeType},
};
use std::io::{Seek, SeekFrom, Write};

impl Measures {
    /// The M range, or `None` when either bound is "no data".
    fn defined_range(&self) -> Option<Interval<f64>> {
        Some(Interval {
            min: self.range.min?,
            max: self.range.max?,
        })
    }
}

/// Writes a main file (`.shp`) record by record, numbering records from 1, and builds its index.
/// [`ShapeWriter::finish`] writes the header; dropping the writer without it leaves the header blank.
#[derive(Debug)]
pub struct ShapeWriter<W> {
    writer: W,
    shape_type: ShapeType,
    extent: Envelope,
    file_len: usize,
    index: Vec<IndexRecord>,
}

impl<W: Write + Seek> ShapeWriter<W> {
    /// Reserves the header, which [`ShapeWriter::finish`] fills in once the extent is known.
    pub fn new(mut writer: W, shape_type: ShapeType) -> Result<Self, FileError> {
        writer.write_all(&[0; 100])?;
        Ok(Self {
            writer,
            shape_type,
            extent: Envelope::default(),
            file_len: 100,
            index: Vec::new(),
        })
    }

    /// Appends a record. Non-null shapes must match the file's shape type.
    pub fn write(&mut self, shape: &Shape) -> Result<(), FileError> {
        let found = shape.shape_type();
        if found != ShapeType::Null && found != self.shape_type {
            return Err(ShapeError::MixedShapeTypes {
                expected: self.shape_type,
                found,
            }
            .into());
        }
        let contents = Vec::<u8>::try_from(shape)?;
        let too_large = |_| ShapeError::TooLarge;
        let entry = IndexRecord {
            offset: i32::try_from(self.file_len / 2).map_err(too_large)?,
            content_length: i32::try_from(contents.len() / 2).map_err(too_large)?,
        };
        let number = i32::try_from(self.index.len() + 1).map_err(too_large)?;
        self.writer.write_all(&number.to_be_bytes())?;
        self.writer.write_all(&entry.content_length.to_be_bytes())?;
        self.writer.write_all(&contents)?;
        self.file_len += 8 + contents.len();
        self.index.push(entry);

        let point = |x: f64, y: f64| Envelope2D {
            xmin: x,
            ymin: y,
            xmax: x,
            ymax: y,
        };
        let value = |v: f64| Interval { min: v, max: v };
        let bounds = match shape {
            _ if shape.is_empty() => None,
            Shape::Null => None,
            Shape::Point(p) => Some((point(p.x, p.y), None, None)),
            Shape::PointM(p) => Some((point(p.x, p.y), None, p.m.map(value))),
            Shape::PointZ(p) => Some((point(p.x, p.y), Some(value(p.z)), p.m.map(value))),
            Shape::MultiPoint(mp) => Some((mp.bbox, None, None)),
            Shape::MultiPointM(mp) => Some((
                mp.xy.bbox,
                None,
                mp.m.as_ref().and_then(Measures::defined_range),
            )),
            Shape::MultiPointZ(mp) => Some((
                mp.xy.bbox,
                Some(mp.z.range),
                mp.m.as_ref().and_then(Measures::defined_range),
            )),
            Shape::PolyLine(p) | Shape::Polygon(p) => Some((p.bbox, None, None)),
            Shape::PolyLineM(p) | Shape::PolygonM(p) => Some((
                p.xy.bbox,
                None,
                p.m.as_ref().and_then(Measures::defined_range),
            )),
            Shape::PolyLineZ(p) | Shape::PolygonZ(p) => Some((
                p.xy.bbox,
                Some(p.z.range),
                p.m.as_ref().and_then(Measures::defined_range),
            )),
            Shape::MultiPatch(p) => Some((
                p.bbox,
                Some(p.z.range),
                p.m.as_ref().and_then(Measures::defined_range),
            )),
        };
        if let Some((bbox, z, m)) = bounds {
            let union = |a: Option<Interval<f64>>, b: Option<Interval<f64>>| match (a, b) {
                (Some(a), Some(b)) => Some(Interval {
                    min: a.min.min(b.min),
                    max: a.max.max(b.max),
                }),
                (a, b) => a.or(b),
            };
            let extent = &mut self.extent;
            extent.xy = Some(match extent.xy {
                Some(b) => Envelope2D {
                    xmin: b.xmin.min(bbox.xmin),
                    ymin: b.ymin.min(bbox.ymin),
                    xmax: b.xmax.max(bbox.xmax),
                    ymax: b.ymax.max(bbox.ymax),
                },
                None => bbox,
            });
            extent.z = union(extent.z, z);
            extent.m = union(extent.m, m);
        }
        Ok(())
    }

    /// Writes the header and returns the flushed underlying writer with the index for a `.shx`.
    pub fn finish(mut self) -> Result<FinishedShapes<W>, FileError> {
        let is_measured = !matches!(
            self.shape_type,
            ShapeType::Null
                | ShapeType::Point
                | ShapeType::PolyLine
                | ShapeType::Polygon
                | ShapeType::MultiPoint
        );
        // Unused ranges are 0.0; a measured file without measures stores "no data", as Esri writes it.
        let m_range = match self.extent.m {
            Some(m) => Interval {
                min: Some(m.min),
                max: Some(m.max),
            },
            None if is_measured => Interval {
                min: None,
                max: None,
            },
            None => Interval {
                min: Some(0.0),
                max: Some(0.0),
            },
        };
        let header = FileHeader {
            file_length: i32::try_from(self.file_len / 2).map_err(|_| ShapeError::TooLarge)?,
            version: 1000,
            shape_type: self.shape_type,
            bbox: self.extent.xy.unwrap_or(Envelope2D {
                xmin: 0.0,
                ymin: 0.0,
                xmax: 0.0,
                ymax: 0.0,
            }),
            z_range: self.extent.z.unwrap_or(Interval { min: 0.0, max: 0.0 }),
            m_range,
        };
        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(&<[u8; 100]>::from(&header))?;
        self.writer.seek(SeekFrom::End(0))?;
        self.writer.flush()?;

        let index_words = 50 + 4 * self.index.len();
        let index = ShapeIndex {
            header: FileHeader {
                file_length: i32::try_from(index_words).map_err(|_| ShapeError::TooLarge)?,
                ..header
            },
            records: self.index,
        };
        Ok(FinishedShapes {
            writer: self.writer,
            index,
        })
    }
}

/// A finished main file and the index describing it, ready to write as the `.shx`.
#[derive(Debug)]
pub struct FinishedShapes<W> {
    pub writer: W,
    pub index: ShapeIndex,
}
