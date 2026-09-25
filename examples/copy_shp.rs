//! Streams every record of one `.shp` file into another, writing its `.shx` index alongside.
//!
//! ```sh
//! cargo run --example copy_shp -- in.shp out.shp
//! ```

use serde_esri::shape::{FileError, FinishedShapes, ShapeReader, ShapeWriter};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

fn main() -> Result<(), FileError> {
    let mut args = std::env::args().skip(1);
    let (input, output) = (args.next().unwrap_or_default(), PathBuf::from(args.next().unwrap_or_default()));

    let reader = ShapeReader::new(BufReader::new(File::open(input)?))?;
    let mut writer = ShapeWriter::new(BufWriter::new(File::create(&output)?), reader.header.shape_type)?;
    for record in reader {
        writer.write(&record?.shape)?;
    }
    let FinishedShapes { index, .. } = writer.finish()?;
    std::fs::write(output.with_extension("shx"), Vec::<u8>::from(&index))?;
    Ok(())
}
