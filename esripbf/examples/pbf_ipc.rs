//! Times decoding PBF query results into Arrow, then writes each batch as an Arrow IPC stream
//! next to its input, so other runtimes can time their side of the handoff.
//!
//! ```sh
//! cargo run --release -p esripbf --example pbf_ipc --features geoarrow -- a.pbf b.pbf
//! ```

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use esripbf::{prost::Message, FeatureCollectionPBuffer};
use std::{fs::File, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in std::env::args().skip(1).map(PathBuf::from) {
        let bytes = std::fs::read(&path)?;
        let mut times = Vec::new();
        let mut batch = None;
        for _ in 0..5 {
            let start = Instant::now();
            let collection = FeatureCollectionPBuffer::decode(bytes.as_slice())?;
            batch = Some(RecordBatch::try_from(collection)?);
            times.push(start.elapsed().as_secs_f64());
        }
        times.sort_by(f64::total_cmp);
        let batch = batch.ok_or("no batch")?;
        println!(
            "{}: {} rows, {:.1} MB: PBF to Arrow {:.1} ms",
            path.display(),
            batch.num_rows(),
            bytes.len() as f64 / 1e6,
            times[times.len() / 2] * 1000.0
        );

        let mut writer = StreamWriter::try_new(
            File::create(path.with_extension("arrows"))?,
            &batch.schema(),
        )?;
        writer.write(&batch)?;
        writer.finish()?;
    }
    Ok(())
}
