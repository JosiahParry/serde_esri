//! Reads shapes as defined by the
//! [ESRI Shapefile Technical Description](https://www.esri.com/content/dam/esrisites/sitecore-archive/Files/Pdfs/library/whitepapers/pdfs/shapefile.pdf)
//! (July 1998).
//!
//! A shape buffer is the contents of one main file record: a little endian
//! shape type followed by that type's geometry. Types keep the layout of the
//! specification: bounding boxes, part start indexes, closing vertices, and Z
//! and M ranges are kept as stored.
//!
//! ```ignore
//! let shape = Shape::try_from(record_contents)?;
//! let shapes = ShapeFile::try_from(shp_bytes)?.collect::<Result<Vec<_>, _>>()?;
//!
//! // Stream records without holding the file in memory.
//! for record in ShapeReader::new(BufReader::new(File::open("file.shp")?))? {
//!     let record = record?;
//! }
//!
//! // Write a shape buffer, or stream records to a file; `finish` fills in the header.
//! let bytes = Vec::<u8>::try_from(&shape)?;
//! let mut writer = ShapeWriter::new(BufWriter::new(File::create("out.shp")?), ShapeType::Polygon)?;
//! writer.write(&shape)?;
//! writer.finish()?;
//! ```
//!
//! Measures smaller than `-1e38` are "no data" and read as `None`. For measured
//! multipoint and multipart types the M section is optional and reads as `None`
//! when the record ends before it.

mod buffer;
pub mod error;
pub mod file;
pub mod index;
pub mod reader;
pub mod types;
pub mod writer;

#[cfg(test)]
mod tests;

