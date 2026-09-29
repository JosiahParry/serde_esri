//! Checks every `.shp` under a directory: reads it, rewrites the `.shp` and `.shx` and
//! compares bytes, and checks random access against sequential reading.
//!
//! ```sh
//! cargo run --example shp_corpus -- path/to/dir
//! ```

use serde_esri::enginex::{geometry::Geometry, shape::writer::EsriShapeBuffer};
use serde_esri::shape::{
    error::{FileError, ShapeError},
    file::{Record, ShapeFile},
    index::ShapeIndex,
    reader::ShapeReader,
    types::Shape,
    writer::{FinishedShapes, ShapeWriter},
};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

fn main() -> Result<(), FileError> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_default());
    let mut paths = Vec::new();
    let mut dirs = vec![dir];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("shp")) {
                paths.push(path);
            }
        }
    }
    paths.sort();

    for path in &paths {
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        println!("{name:<45} {}", check(path));
    }
    Ok(())
}

/// One line summarizing every check on a file.
fn check(path: &Path) -> String {
    let shp = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => return format!("unreadable: {e}"),
    };
    let file = match ShapeFile::try_from(shp.as_slice()) {
        Ok(file) => file,
        Err(e) => return format!("HEADER ERROR {e}"),
    };
    let shape_type = file.header.shape_type;
    let records = match file.collect::<Result<Vec<Record>, _>>() {
        Ok(records) => records,
        Err(e) => return format!("{shape_type:?}: READ ERROR {e}"),
    };
    let mut line = format!("{shape_type:?} x{}", records.len());

    let mut writer = match ShapeWriter::new(Cursor::new(Vec::new()), shape_type) {
        Ok(writer) => writer,
        Err(e) => return format!("{line}; WRITE ERROR {e}"),
    };
    for record in &records {
        if let Err(e) = writer.write(&record.shape) {
            return format!("{line}; WRITE ERROR {e}");
        }
    }
    let FinishedShapes { writer, index } = match writer.finish() {
        Ok(finished) => finished,
        Err(e) => return format!("{line}; WRITE ERROR {e}"),
    };
    line += &format!("; shp {}", compare(&shp, &writer.into_inner()));

    let shx = std::fs::read(path.with_extension("shx")).ok();
    match &shx {
        Some(shx) => line += &format!("; shx {}", compare(shx, &Vec::<u8>::from(&index))),
        None => line += "; no shx",
    }

    let original_index = shx.as_deref().map(ShapeIndex::try_from);
    let random_access = [None, original_index]
        .into_iter()
        .map(|index| random_access(&shp, &records, index))
        .collect::<Vec<_>>()
        .join("/");
    line += &format!("; random {random_access}");

    // Converts each record into the engine and back, comparing the shape buffers.
    // Also writes each geometry as an engine shape buffer and reads it back.
    let mut failures = std::collections::BTreeMap::<String, usize>::new();
    let (mut converted, mut unchanged, mut engine_buffers) = (0, 0, 0);
    for record in &records {
        let geometry = match Geometry::try_from(record.shape.clone()) {
            Ok(geometry) => geometry,
            Err(e) => {
                *failures.entry(e.to_string()).or_default() += 1;
                continue;
            }
        };
        converted += 1;
        let back = Shape::try_from(&geometry).map(|shape| Vec::<u8>::try_from(&shape));
        if let Ok(Ok(bytes)) = back {
            unchanged += usize::from(Ok(bytes) == Vec::<u8>::try_from(&record.shape));
        }
        let buffer = EsriShapeBuffer::try_from(&geometry);
        let read_back = buffer
            .ok()
            .and_then(|buffer| Option::<Geometry>::try_from(buffer.as_shape()).ok());
        engine_buffers += usize::from(format!("{read_back:?}") == format!("{:?}", Some(Some(geometry))));
    }
    line + &format!(
        "; enginex {converted} ok, {unchanged} unchanged, {engine_buffers} engine buffers same {failures:?}"
    )
}

/// "same", or where and how the rewritten bytes first differ.
fn compare(original: &[u8], rewritten: &[u8]) -> String {
    if original == rewritten {
        return "same".into();
    }
    let at = original
        .iter()
        .zip(rewritten)
        .position(|(a, b)| a != b)
        .unwrap_or(original.len().min(rewritten.len()));
    let region = if at < 100 { "header" } else { "records" };
    format!(
        "DIFF at byte {at} ({region}), lengths {} vs {}",
        original.len(),
        rewritten.len()
    )
}

/// Reads each record by position, without an index or with the original `.shx`.
fn random_access(
    shp: &[u8],
    records: &[Record],
    index: Option<Result<ShapeIndex, ShapeError>>,
) -> String {
    let label = if index.is_some() { "shx" } else { "scan" };
    let reader = ShapeReader::new(Cursor::new(shp));
    let mut reader = match (reader, index) {
        (Ok(reader), None) => reader,
        (Ok(reader), Some(Ok(index))) => reader.with_index(index),
        (Ok(_), Some(Err(e))) => return format!("{label} BAD SHX {e}"),
        (Err(e), _) => return format!("{label} ERROR {e}"),
    };
    for (i, expected) in records.iter().enumerate().rev() {
        match reader.read_nth(i) {
            // Compared as bytes, since NaN coordinates never compare equal.
            Some(Ok(record))
                if record.number == expected.number
                    && Vec::<u8>::try_from(&record.shape) == Vec::<u8>::try_from(&expected.shape) => {}
            Some(Ok(_)) => return format!("{label} MISMATCH at {i}"),
            Some(Err(e)) => return format!("{label} ERROR at {i}: {e}"),
            None => return format!("{label} MISSING {i}"),
        }
    }
    format!("{label} ok")
}
