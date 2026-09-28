//! Times PBF query results into Arrow: the protobuf decode alone, then decode and conversion.
//!
//! ```sh
//! cargo run --release -p esripbf --example bench_pbf_arrow --features geoarrow -- a.pbf b.pbf
//! ```

use arrow_array::RecordBatch;
use esripbf::{prost::Message, FeatureCollectionPBuffer};
use std::{path::PathBuf, time::Instant};

/// The median of five runs, in milliseconds.
fn median_ms<T>(
    mut run: impl FnMut() -> Result<T, Box<dyn std::error::Error>>,
) -> Result<(f64, T), Box<dyn std::error::Error>> {
    let mut times = Vec::new();
    let mut last = None;
    for _ in 0..5 {
        let start = Instant::now();
        last = Some(run()?);
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(f64::total_cmp);
    Ok((times[times.len() / 2], last.ok_or("no runs")?))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in std::env::args().skip(1).map(PathBuf::from) {
        let bytes = std::fs::read(&path)?;
        let (decode, _) = median_ms(|| Ok(FeatureCollectionPBuffer::decode(bytes.as_slice())?))?;
        let (total, batch) = median_ms(|| {
            let collection = FeatureCollectionPBuffer::decode(bytes.as_slice())?;
            Ok(RecordBatch::try_from(collection)?)
        })?;
        println!(
            "{}: {} rows, {} columns, {:.1} MB: decode {decode:.1} ms, decode and convert {total:.1} ms",
            path.file_name().map_or(path.display().to_string(), |n| n.to_string_lossy().into_owned()),
            batch.num_rows(),
            batch.num_columns(),
            bytes.len() as f64 / 1e6,
        );
    }
    Ok(())
}
