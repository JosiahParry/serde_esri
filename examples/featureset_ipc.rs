//! Writes a synthetic polygon FeatureSet as JSON, times streaming it into Arrow, and writes the
//! result as an Arrow IPC stream, so other runtimes can time their side of the handoff.
//!
//! ```sh
//! cargo run --release --example featureset_ipc --features geoarrow -- 50000 64 out/
//! ```

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use serde_esri::arrow_compat::FeatureSetJson;
use std::{fs::File, path::PathBuf, time::Instant};

mod common;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut number = |default: usize| -> Result<usize, String> {
        args.next().map_or(Ok(default), |a| a.parse().map_err(|e| format!("{e}")))
    };
    let (features, vertices) = (number(50_000)?, number(64)?);
    let dir = PathBuf::from(std::env::args().nth(3).unwrap_or_else(|| ".".into()));

    let json = common::feature_set(features, vertices);
    std::fs::write(dir.join("featureset.json"), &json).map_err(|e| e.to_string())?;

    let mut times = Vec::new();
    let mut batch = None;
    for _ in 0..5 {
        let start = Instant::now();
        batch = Some(RecordBatch::try_from(FeatureSetJson(json.as_bytes())).map_err(|e| e.to_string())?);
        times.push(start.elapsed().as_secs_f64());
    }
    times.sort_by(f64::total_cmp);
    let batch = batch.ok_or("no batch")?;
    println!(
        "{features} polygons x {vertices} vertices, {:.1} MB: JSON to Arrow {:.1} ms",
        json.len() as f64 / 1e6,
        times[times.len() / 2] * 1000.0
    );

    let file = File::create(dir.join("featureset.arrows")).map_err(|e| e.to_string())?;
    let mut writer = StreamWriter::try_new(file, &batch.schema()).map_err(|e| e.to_string())?;
    writer.write(&batch).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}
