use super::*;
use arrow_array::{cast::AsArray, types::Float64Type};
use prost::Message;
use serde_esri::arrow_compat::FeatureSetJson;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture(name: &str) -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(format!("{}/tests/data/{name}", env!("CARGO_MANIFEST_DIR")))
}

/// The PBF batch has the JSON batch's schema, rows, and attribute columns.
fn matches_json(name: &str) -> TestResult {
    let collection = FeatureCollectionPBuffer::decode(fixture(&format!("{name}.pbf"))?.as_slice())?;
    let pbf = RecordBatch::try_from(collection)?;
    let json = RecordBatch::try_from(FeatureSetJson(&fixture(&format!("{name}.json"))?))?;

    assert_eq!(pbf.schema(), json.schema(), "{name}: schema");
    assert_eq!(pbf.num_rows(), json.num_rows(), "{name}: rows");
    for (i, field) in json.schema().fields().iter().enumerate() {
        let (a, b) = (pbf.column(i), json.column(i));
        let floats = (
            a.as_primitive_opt::<Float64Type>(),
            b.as_primitive_opt::<Float64Type>(),
        );
        if field.name() == "geometry" {
            continue;
        }
        // The PBF holds the exact double; the service's JSON text can round the last bit.
        if let (Some(a), Some(b)) = floats {
            for (x, y) in a.iter().zip(b.iter()) {
                let close =
                    |x: f64, y: f64| (x - y).abs() <= 4.0 * f64::EPSILON * x.abs().max(y.abs());
                assert!(
                    x.zip(y).map_or(x == y, |(x, y)| close(x, y)),
                    "{name} {}: {x:?} vs {y:?}",
                    field.name()
                );
            }
        } else {
            assert_eq!(a, b, "{name}: column {}", field.name());
        }
    }
    Ok(())
}

#[test]
fn batches_match_json() -> TestResult {
    for name in [
        "points",
        "lines",
        "polygons",
        "lines_z",
        "lines_m",
        "lines_zm",
        "no_geometry",
        "table",
    ] {
        matches_json(name)?;
    }
    Ok(())
}

#[test]
fn counts_are_not_feature_results() {
    let result = RecordBatch::try_from(FeatureCollectionPBuffer::default());
    assert!(matches!(
        result,
        Err(PbfToArrowError::Pbf(FromPbfError::NotFeatureResult))
    ));
}
