//! Times FeatureSet JSON to Arrow on a synthetic polygon layer: parsing a `FeatureSet` then
//! converting it, against streaming the JSON straight into Arrow.
//!
//! ```sh
//! cargo run --release --example bench_featureset_arrow --features geoarrow -- 50000 64
//! ```

use arrow_array::RecordBatch;
use serde_esri::{arrow_compat::FeatureSetJson, features::FeatureSet};
use std::{fmt::Write, time::Instant};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1).map(|a| a.parse::<usize>());
    let features = args.next().unwrap_or(Ok(50_000)).map_err(|e| e.to_string())?;
    let vertices = args.next().unwrap_or(Ok(64)).map_err(|e| e.to_string())?;
    let json = feature_set(features, vertices);
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
        let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        Ok(value["features"].as_array().map_or(0, Vec::len))
    })?;
    time("FeatureSet then RecordBatch", &|| {
        let feature_set: FeatureSet<2> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let batch = RecordBatch::try_from(&feature_set).map_err(|e| e.to_string())?;
        Ok(batch.num_rows())
    })?;
    time("FeatureSetJson streamed", &|| {
        let batch = RecordBatch::try_from(FeatureSetJson(json.as_bytes())).map_err(|e| e.to_string())?;
        Ok(batch.num_rows())
    })?;
    Ok(())
}

/// A FeatureSet of `features` square-ish polygons with `vertices` vertices and five attributes.
fn feature_set(features: usize, vertices: usize) -> String {
    let mut json = String::from(
        r#"{"objectIdFieldName":"OBJECTID","geometryType":"esriGeometryPolygon","spatialReference":{"wkid":4326},"fields":[{"name":"OBJECTID","type":"esriFieldTypeOID"},{"name":"NAME","type":"esriFieldTypeString"},{"name":"POP","type":"esriFieldTypeInteger"},{"name":"AREA","type":"esriFieldTypeDouble"},{"name":"UPDATED","type":"esriFieldTypeDate"}],"features":["#,
    );
    for i in 0..features {
        if i > 0 {
            json.push(',');
        }
        let (cx, cy) = ((i % 1000) as f64 * 0.01 - 5.0, (i / 1000) as f64 * 0.01 + 30.0);
        let _ = write!(
            json,
            r#"{{"attributes":{{"OBJECTID":{i},"NAME":"feature {i}","POP":{},"AREA":{:.6},"UPDATED":{}}},"geometry":{{"rings":[["#,
            i * 7 % 100_000,
            i as f64 * 0.37,
            1_700_000_000_000_i64 + i as i64
        );
        // A clockwise ring, closed by repeating the first vertex.
        for v in 0..=vertices {
            let angle = -((v % vertices) as f64) / vertices as f64 * std::f64::consts::TAU;
            let _ = write!(
                json,
                "{}[{:.8},{:.8}]",
                if v > 0 { "," } else { "" },
                cx + 0.004 * angle.cos(),
                cy + 0.004 * angle.sin()
            );
        }
        json.push_str("]]}}");
    }
    json.push_str("]}");
    json
}
