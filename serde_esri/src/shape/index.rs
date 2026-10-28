//! The index file (`.shx`), which locates each main file record.

use crate::shape::{buffer::Reader, error::ShapeError, file::FileHeader};

/// One index file (`.shx`) record: where a main file record starts and its content length.
/// Both are in 16-bit words; the first record's offset is 50, just past the header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexRecord {
    pub offset: i32,
    pub content_length: i32,
}

impl IndexRecord {
    /// Byte position of the record header in the main file.
    pub(super) fn start(self) -> Result<u64, ShapeError> {
        u64::try_from(self.offset)
            .map(|words| words * 2)
            .map_err(|_| ShapeError::Corrupted)
    }

    /// Byte position just past the record's contents.
    pub(super) fn end(self) -> Result<u64, ShapeError> {
        let content_length = u64::try_from(self.content_length).map_err(|_| ShapeError::Corrupted)?;
        Ok(self.start()? + 8 + content_length * 2)
    }
}

/// An index file (`.shx`): the main file header, with its own file length, and one
/// [`IndexRecord`] per main file record.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeIndex {
    pub header: FileHeader,
    pub records: Vec<IndexRecord>,
}

/// Reads an index file.
impl TryFrom<&[u8]> for ShapeIndex {
    type Error = ShapeError;

    fn try_from(file: &[u8]) -> Result<Self, Self::Error> {
        let header = FileHeader::try_from(file)?;
        let entries = file.get(100..).ok_or(ShapeError::UnexpectedEof)?.chunks_exact(8);
        if !entries.remainder().is_empty() {
            return Err(ShapeError::UnexpectedEof);
        }
        let records = entries
            .map(|entry| {
                let mut r = Reader(entry);
                Ok(IndexRecord {
                    offset: r.i32_be()?,
                    content_length: r.i32_be()?,
                })
            })
            .collect::<Result<_, ShapeError>>()?;
        Ok(Self { header, records })
    }
}

/// Writes an index file.
impl From<&ShapeIndex> for Vec<u8> {
    fn from(index: &ShapeIndex) -> Self {
        let mut bytes = Vec::with_capacity(100 + 8 * index.records.len());
        bytes.extend(<[u8; 100]>::from(&index.header));
        for record in &index.records {
            bytes.extend(record.offset.to_be_bytes());
            bytes.extend(record.content_length.to_be_bytes());
        }
        bytes
    }
}
