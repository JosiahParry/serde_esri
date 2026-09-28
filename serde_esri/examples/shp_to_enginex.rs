//! Reads a `.shp` into engine geometries, skipping null records.
//!
//! ```sh
//! cargo run --example shp_to_enginex -- path/to/file.shp
//! ```

use serde_esri::{
    enginex::Geometry,
    shape::{Shape, ShapeReader},
};
use std::{error::Error, fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).unwrap_or_default();
    let reader = ShapeReader::new(BufReader::new(File::open(path)?))?;

    let mut geometries: Vec<Geometry> = Vec::new();
    for record in reader {
        let shape = record?.shape;
        if shape != Shape::Null {
            geometries.push(Geometry::try_from(shape)?);
        }
    }

    println!("{} geometries", geometries.len());
    if let Some(first) = geometries.first() {
        println!("{first:?}");
    }
    Ok(())
}
