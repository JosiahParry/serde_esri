use super::*;
use arrow_array::{cast::AsArray, types::Float64Type, Array};
use prost::Message;
use serde_esri::arrow_compat::FeatureSetJson;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture(name: &str) -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(format!("{}/tests/data/{name}", env!("CARGO_MANIFEST_DIR")))
}

/// A geometry column's list offsets, outermost first, and its coordinate values.
fn layout(array: &dyn Array) -> (Vec<Vec<i32>>, Vec<f64>) {
    if let Some(list) = array.as_list_opt::<i32>() {
        let (mut offsets, values) = layout(list.values().as_ref());
        offsets.insert(0, list.value_offsets().to_vec());
        (offsets, values)
    } else if let Some(fixed) = array.as_fixed_size_list_opt() {
        layout(fixed.values().as_ref())
    } else {
        let values = array.as_primitive_opt::<Float64Type>();
        (
            Vec::new(),
            values.map(|v| v.values().to_vec()).unwrap_or_default(),
        )
    }
}

/// The PBF batch has the JSON batch's schema, rows, and columns, with coordinates within the
/// quantization step.
fn matches_json(name: &str) -> TestResult {
    let collection = FeatureCollectionPBuffer::decode(fixture(&format!("{name}.pbf"))?.as_slice())?;
    let Some(Results::FeatureResult(result)) =
        collection.query_result.clone().and_then(|q| q.results)
    else {
        return Err("not a feature result".into());
    };
    let scale = result.transform.and_then(|t| t.scale).unwrap_or_default();
    let step = [scale.x_scale, scale.y_scale, scale.z_scale, scale.m_scale]
        .into_iter()
        .fold(0.0, f64::max);
    let pbf = RecordBatch::try_from(collection)?;
    let json = RecordBatch::try_from(FeatureSetJson(&fixture(&format!("{name}.json"))?))?;

    assert_eq!(pbf.schema(), json.schema(), "{name}: schema");
    assert_eq!(pbf.num_rows(), json.num_rows(), "{name}: rows");
    for (i, field) in json.schema().fields().iter().enumerate() {
        let (a, b) = (pbf.column(i), json.column(i));
        if field.name() == "geometry" {
            let ((a_offsets, a_coords), (b_offsets, b_coords)) = (layout(a), layout(b));
            assert_eq!(a_offsets, b_offsets, "{name}: geometry offsets");
            assert_eq!(
                a.logical_nulls(),
                b.logical_nulls(),
                "{name}: geometry nulls"
            );
            assert_eq!(a_coords.len(), b_coords.len(), "{name}: coordinate count");
            for (x, y) in a_coords.iter().zip(&b_coords) {
                assert!(
                    (x - y).abs() <= step || (x.is_nan() && y.is_nan()),
                    "{name}: {x} vs {y}"
                );
            }
            continue;
        }
        let floats = (
            a.as_primitive_opt::<Float64Type>(),
            b.as_primitive_opt::<Float64Type>(),
        );
        // The PBF holds the exact double; the service's JSON text can round the last bit.
        if let (Some(a), Some(b)) = floats {
            for (x, y) in a.iter().zip(b.iter()) {
                let close =
                    |x: f64, y: f64| (x - y).abs() <= 4.0 * f64::EPSILON * x.abs().max(y.abs());
                let same = x.zip(y).map_or(x == y, |(x, y)| close(x, y));
                assert!(same, "{name} {}: {x:?} vs {y:?}", field.name());
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
