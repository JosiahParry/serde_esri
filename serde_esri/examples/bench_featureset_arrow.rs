//! Times FeatureSet JSON to Arrow on a synthetic polygon layer: parsing a `FeatureSet` then
//! converting it, against streaming the JSON straight into Arrow.
//!
//! ```sh
//! cargo run --release --example bench_featureset_arrow --features geoarrow -- 50000 64
//! ```

use arrow_array::RecordBatch;
use serde_esri::{arrow_compat::json::FeatureSetJson, features::FeatureSet};
use std::time::Instant;

mod common;

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1).map(|a| a.parse::<usize>());
    let features = args.next().unwrap_or(Ok(50_000)).map_err(|e| e.to_string())?;
    let vertices = args.next().unwrap_or(Ok(64)).map_err(|e| e.to_string())?;
    let json = common::feature_set(features, vertices);
    let megabytes = json.len() as f64 / 1_000_000.0;
    println!("{features} polygons, {vertices} vertices each, {megabytes:.1} MB");

    let time = |name: &str, run: &dyn Fn() -> Result<usize, String>| -> Result<(), String> {
        let mut times = Vec::new();
        let mut rows = 0;
        for _ in 0..5 {
            let start = Instant::now();
            rows = run()?;
            times.push(start.elapsed().as_secs_f64());
        }
        times.sort_by(f64::total_cmp);
        let median = times[times.len() / 2];
        println!(
            "{name:<34} {:>8.1} ms  {:>7.1} MB/s  ({rows} rows)",
            median * 1000.0,
            megabytes / median
        );
        Ok(())
    };

    time("serde_json::Value (parse only)", &|| {
        let value = serde_json::from_str::<serde_json::Value>(&json).map_err(|e| e.to_string())?;
        Ok(value["features"].as_array().map_or(0, Vec::len))
    })?;
    time("FeatureSet then RecordBatch", &|| {
        let feature_set = serde_json::from_str::<FeatureSet<2>>(&json).map_err(|e| e.to_string())?;
        let batch = RecordBatch::try_from(&feature_set).map_err(|e| e.to_string())?;
        Ok(batch.num_rows())
    })?;
    time("FeatureSetJson streamed", &|| {
        let batch = RecordBatch::try_from(FeatureSetJson(json.as_bytes())).map_err(|e| e.to_string())?;
        Ok(batch.num_rows())
    })?;
    Ok(())
}

