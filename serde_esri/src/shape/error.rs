//! Errors from reading and writing shapes and shapefiles.

use crate::shape::types::ShapeType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeError {
    UnexpectedEof,
    InvalidFileCode(i32),
    InvalidShapeType(i32),
    InvalidPartType(i32),
    Corrupted,
    MixedShapeTypes { expected: ShapeType, found: ShapeType },
    TooLarge,
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

/// Error from [`ShapeReader`](crate::shape::reader::ShapeReader) or [`ShapeWriter`](crate::shape::writer::ShapeWriter): I/O failed, or the shapes are invalid.
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
