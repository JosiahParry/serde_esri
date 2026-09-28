//! A synthetic polygon FeatureSet shared by the benchmark examples.

use std::fmt::Write;

/// A FeatureSet of `features` square-ish polygons with `vertices` vertices and five attributes.
pub fn feature_set(features: usize, vertices: usize) -> String {
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
