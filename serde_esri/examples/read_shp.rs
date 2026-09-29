//! Streams every record of a `.shp` file and prints a summary.
//!
//! ```sh
//! cargo run --example read_shp -- path/to/file.shp
//! ```

use serde_esri::shape::{error::FileError, reader::ShapeReader};
use std::{fs::File, io::BufReader};

fn main() -> Result<(), FileError> {
    let path = std::env::args().nth(1).unwrap_or_default();
    let reader = ShapeReader::new(BufReader::new(File::open(&path)?))?;
    println!("{:?}", reader.header);

    let mut count = 0;
    for record in reader {
        let record = record?;
        if count < 3 {
            println!("{record:?}");
        }
        count += 1;
    }
    println!("{count} records");
    Ok(())
}
