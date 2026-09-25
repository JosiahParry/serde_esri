//! Streaming a main file from a buffered reader, with random access when it can seek.

use crate::shape::{
    file::RecordHeader, FileError, FileHeader, IndexRecord, Record, Shape, ShapeError, ShapeIndex,
};
use std::io::{BufRead, Read, Seek, SeekFrom};

/// Streams a main file (`.shp`) from a buffered reader, one record at a time.
/// Iteration stops after the first error.
///
/// With a seekable reader, [`ShapeReader::seek`] and [`ShapeReader::read_nth`] jump to a record.
/// An index from [`ShapeReader::with_index`] locates records directly; without one, record
/// headers are skipped through. Record locations are remembered as they are found.
#[derive(Debug)]
pub struct ShapeReader<R> {
    pub header: FileHeader,
    reader: R,
    contents: Vec<u8>,
    is_done: bool,
    /// Locations of the records found so far, in record order.
    index: Vec<IndexRecord>,
    /// Position of the record `next()` reads.
    next_record: usize,
    position: u64,
}

impl<R: BufRead> ShapeReader<R> {
    /// Reads the main file header, leaving the records to iteration.
    pub fn new(mut reader: R) -> Result<Self, FileError> {
        let mut header = [0u8; 100];
        reader.read_exact(&mut header)?;
        Ok(Self {
            header: FileHeader::try_from(header.as_slice())?,
            reader,
            contents: Vec::new(),
            is_done: false,
            index: Vec::new(),
            next_record: 0,
            position: 100,
        })
    }

    /// Uses a `.shx` index to locate records instead of skipping through the file.
    pub fn with_index(mut self, index: ShapeIndex) -> Self {
        self.index = index.records;
        self
    }

    /// Reads one record, reusing the contents buffer between records.
    fn read_record(&mut self) -> Result<Record, FileError> {
        let mut header = [0u8; 8];
        self.reader.read_exact(&mut header)?;
        let header = RecordHeader::try_from(header)?;

        // Grows with the bytes actually read, so a corrupt length cannot force a huge allocation.
        self.contents.clear();
        (&mut self.reader)
            .take(header.content_len as u64)
            .read_to_end(&mut self.contents)?;
        if self.contents.len() < header.content_len {
            return Err(ShapeError::UnexpectedEof.into());
        }

        if self.next_record == self.index.len() {
            let too_large = |_| ShapeError::TooLarge;
            self.index.push(IndexRecord {
                offset: i32::try_from(self.position / 2).map_err(too_large)?,
                content_length: i32::try_from(header.content_len / 2).map_err(too_large)?,
            });
        }
        self.next_record += 1;
        self.position += 8 + header.content_len as u64;

        Ok(Record {
            number: header.number,
            shape: Shape::try_from(self.contents.as_slice())?,
        })
    }
}

impl<R: BufRead + Seek> ShapeReader<R> {
    /// Positions the reader so iteration continues from the record at `n`, counting from 0.
    /// `n` may equal the record count, leaving nothing to read.
    pub fn seek(&mut self, n: usize) -> Result<(), FileError> {
        while self.index.len() <= n {
            let start = self.index.last().map_or(Ok(100), |r| r.end())?;
            self.reader.seek(SeekFrom::Start(start))?;
            if self.reader.fill_buf()?.is_empty() {
                if self.index.len() < n {
                    return Err(ShapeError::NoSuchRecord(n).into());
                }
                self.next_record = n;
                self.position = start;
                self.is_done = false;
                return Ok(());
            }
            let mut header = [0u8; 8];
            self.reader.read_exact(&mut header)?;
            let header = RecordHeader::try_from(header)?;
            let too_large = |_| ShapeError::TooLarge;
            self.index.push(IndexRecord {
                offset: i32::try_from(start / 2).map_err(too_large)?,
                content_length: i32::try_from(header.content_len / 2).map_err(too_large)?,
            });
        }

        let start = self.index.get(n).ok_or(ShapeError::NoSuchRecord(n))?.start()?;
        self.reader.seek(SeekFrom::Start(start))?;
        self.next_record = n;
        self.position = start;
        self.is_done = false;
        Ok(())
    }

    /// Reads the record at `n`, counting from 0, or `None` past the last record.
    /// Iteration then continues from the record after it.
    pub fn read_nth(&mut self, n: usize) -> Option<Result<Record, FileError>> {
        match self.seek(n) {
            Ok(()) => self.next(),
            Err(FileError::Shape(ShapeError::NoSuchRecord(_))) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<R: BufRead> Iterator for ShapeReader<R> {
    type Item = Result<Record, FileError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }
        let record = match self.reader.fill_buf() {
            Ok([]) => {
                self.is_done = true;
                return None;
            }
            Ok(_) => self.read_record(),
            Err(e) => Err(e.into()),
        };
        self.is_done = record.is_err();
        Some(record)
    }
}
