//! Errors from reading and writing shapes and shapefiles.

use crate::shape::ShapeType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeError {
    /// The buffer ended before the shape was fully read.
    UnexpectedEof,
    /// The main file header does not start with file code 9994.
    InvalidFileCode(i32),
    InvalidShapeType(i32),
    InvalidPartType(i32),
    /// A count is negative or disagrees with the points, or a part index is out of order or range.
    Corrupted,
    /// A non-null record's type differs from the file's shape type.
    MixedShapeTypes { expected: ShapeType, found: ShapeType },
    /// A count or length does not fit the format's 32-bit integers.
    TooLarge,
    /// The file has fewer records than the position sought.
    NoSuchRecord(usize),
}

impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeError::UnexpectedEof => write!(f, "unexpected end of shape buffer"),
            ShapeError::InvalidFileCode(code) => write!(f, "invalid shapefile file code {code}"),
            ShapeError::InvalidShapeType(t) => write!(f, "invalid shape type {t}"),
            ShapeError::InvalidPartType(t) => write!(f, "invalid multipatch part type {t}"),
            ShapeError::Corrupted => write!(f, "corrupted shape"),
            ShapeError::MixedShapeTypes { expected, found } => {
                write!(f, "cannot write a {found:?} record to a {expected:?} file")
            }
            ShapeError::TooLarge => write!(f, "shape exceeds the format's size limits"),
            ShapeError::NoSuchRecord(n) => write!(f, "no record at position {n}"),
        }
    }
}

impl std::error::Error for ShapeError {}

/// Error from [`ShapeReader`](crate::shape::ShapeReader) or [`ShapeWriter`](crate::shape::ShapeWriter): I/O failed, or the shapes are invalid.
#[derive(Debug)]
pub enum FileError {
    Io(std::io::Error),
    Shape(ShapeError),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::Io(e) => write!(f, "{e}"),
            FileError::Shape(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for FileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FileError::Io(e) => Some(e),
            FileError::Shape(e) => Some(e),
        }
    }
}

/// A reader running out of bytes is a truncated shapefile.
impl From<std::io::Error> for FileError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::UnexpectedEof => FileError::Shape(ShapeError::UnexpectedEof),
            _ => FileError::Io(e),
        }
    }
}

impl From<ShapeError> for FileError {
    fn from(e: ShapeError) -> Self {
        FileError::Shape(e)
    }
}
