//! The main file (`.shp`): its header and records, held in memory.

use crate::shape::{
    buffer::{Reader, Writer},
    error::ShapeError,
    types::{BoundingBox, Range, Shape, ShapeType},
};

/// The 100 byte main file (`.shp`) header.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileHeader {
    /// Total file length in 16-bit words, including the header.
    pub file_length: i32,
    pub version: i32,
    pub shape_type: ShapeType,
    pub bbox: BoundingBox,
    pub z_range: Range<f64>,
    pub m_range: Range<Option<f64>>,
}

/// Reads the first 100 bytes as a main file header.
impl TryFrom<&[u8]> for FileHeader {
    type Error = ShapeError;

    fn try_from(file: &[u8]) -> Result<Self, Self::Error> {
        let mut r = Reader(file);
        let file_code = r.i32_be()?;
        if file_code != 9994 {
            return Err(ShapeError::InvalidFileCode(file_code));
        }
        r.take::<20>()?; // unused
        Ok(FileHeader {
            file_length: r.i32_be()?,
            version: r.i32()?,
            shape_type: ShapeType::try_from(r.i32()?)?,
            bbox: r.bbox()?,
            z_range: Range {
                min: r.f64()?,
                max: r.f64()?,
            },
            m_range: Range {
                min: r.measure()?,
                max: r.measure()?,
            },
        })
    }
}

/// The 8 byte big endian header that precedes each record's contents.
pub(super) struct RecordHeader {
    pub(super) number: i32,
    pub(super) content_len: usize,
}

/// Content length is stored in 16-bit words.
impl TryFrom<[u8; 8]> for RecordHeader {
    type Error = ShapeError;

    fn try_from(bytes: [u8; 8]) -> Result<Self, Self::Error> {
        let [n0, n1, n2, n3, l0, l1, l2, l3] = bytes;
        let words = usize::try_from(i32::from_be_bytes([l0, l1, l2, l3]))
            .map_err(|_| ShapeError::Corrupted)?;
        Ok(Self {
            number: i32::from_be_bytes([n0, n1, n2, n3]),
            content_len: words * 2,
        })
    }
}

/// One main file record.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    /// Record numbers begin at 1.
    pub number: i32,
    pub shape: Shape,
}

/// A main file (`.shp`) held in memory: its header and a cursor over its records.
/// Iteration stops after the first error.
#[derive(Clone, Debug)]
pub struct ShapeFile<'a> {
    pub header: FileHeader,
    records: &'a [u8],
}

/// Reads the main file header, leaving the records to iteration.
impl<'a> TryFrom<&'a [u8]> for ShapeFile<'a> {
    type Error = ShapeError;

    fn try_from(file: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            header: FileHeader::try_from(file)?,
            records: file.get(100..).ok_or(ShapeError::UnexpectedEof)?,
        })
    }
}

impl Iterator for ShapeFile<'_> {
    type Item = Result<Record, ShapeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.records.is_empty() {
            return None;
        }
        let mut r = Reader(self.records);
        let record = r.take::<8>().and_then(RecordHeader::try_from).and_then(|header| {
            let (contents, rest) = r
                .0
                .split_at_checked(header.content_len)
                .ok_or(ShapeError::UnexpectedEof)?;
            r.0 = rest;
            Ok(Record {
                number: header.number,
                shape: Shape::try_from(contents)?,
            })
        });
        self.records = if record.is_ok() { r.0 } else { &[] };
        Some(record)
    }
}

impl From<&FileHeader> for [u8; 100] {
    fn from(header: &FileHeader) -> Self {
        let mut w = Writer(Vec::with_capacity(100));
        w.0.extend(9994_i32.to_be_bytes());
        w.0.extend([0; 20]);
        w.0.extend(header.file_length.to_be_bytes());
        w.i32(header.version);
        w.i32(header.shape_type as i32);
        w.bbox(&header.bbox);
        w.f64(header.z_range.min);
        w.f64(header.z_range.max);
        w.measure(header.m_range.min);
        w.measure(header.m_range.max);
        let mut bytes = [0; 100];
        bytes.copy_from_slice(&w.0);
        bytes
    }
}
