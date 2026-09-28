use super::*;
use crate::{feature_collection_p_buffer::query_result::Results, FeatureCollectionPBuffer};
use prost::Message;
use serde_esri::{
    features::{EsriValue, FeatureSet},
    geometry::EsriGeometry,
    sqltype::SqlType,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture(name: &str) -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(format!("{}/tests/data/{name}", env!("CARGO_MANIFEST_DIR")))
}

/// Each part's coordinates, a point being one part of one coordinate.
fn parts<const N: usize>(geometry: &EsriGeometry<N>) -> Vec<Vec<Vec<f64>>> {
    match geometry {
        EsriGeometry::Point(p) => {
            let coord = [Some(p.x), Some(p.y), p.z, p.m]
                .into_iter()
                .flatten()
                .collect();
            vec![vec![coord]]
        }
        EsriGeometry::MultiPoint(mp) => vec![mp.points.iter().map(|c| c.0.to_vec()).collect()],
        EsriGeometry::Polyline(pl) => pl
            .paths
            .iter()
            .map(|path| path.0.iter().map(|c| c.0.to_vec()).collect())
            .collect(),
        EsriGeometry::Polygon(pg) => pg
            .rings
            .iter()
            .map(|ring| ring.0.iter().map(|c| c.0.to_vec()).collect())
            .collect(),
        EsriGeometry::Envelope(_) => Vec::new(),
    }
}

fn same_value(a: &EsriValue, b: &EsriValue) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => (x - y).abs() <= 4.0 * f64::EPSILON * x.abs().max(y.abs()),
        _ => a == b,
    }
}

/// Decodes `name.pbf` and checks it against `name.json`, the same query as Esri JSON.
fn matches_json<const N: usize>(name: &str) -> TestResult {
    let collection = FeatureCollectionPBuffer::decode(fixture(&format!("{name}.pbf"))?.as_slice())?;
    let Some(Results::FeatureResult(result)) =
        collection.query_result.clone().and_then(|q| q.results)
    else {
        return Err("not a feature result".into());
    };
    let scale = result
        .transform
        .clone()
        .and_then(|t| t.scale)
        .unwrap_or_default();
    let mut tolerance = vec![scale.x_scale, scale.y_scale];
    tolerance.extend(result.has_z.then_some(scale.z_scale));
    tolerance.extend(result.has_m.then_some(scale.m_scale));

    let pbf = FeatureSet::<N>::try_from(collection)?;
    let json: FeatureSet<N> = serde_json::from_slice(&fixture(&format!("{name}.json"))?)?;

    assert_eq!(pbf.geometryType, json.geometryType, "{name}: geometry type");
    assert_eq!(
        pbf.spatialReference, json.spatialReference,
        "{name}: spatial reference"
    );
    assert_eq!(pbf.hasZ, json.hasZ, "{name}: hasZ");
    assert_eq!(pbf.hasM, json.hasM, "{name}: hasM");
    assert_eq!(
        pbf.objectIdFieldName, json.objectIdFieldName,
        "{name}: object id field"
    );
    let describe = |fs: &FeatureSet<N>| -> Vec<(String, String, Option<SqlType>)> {
        let fields = fs.fields.iter().flatten();
        fields
            .map(|f| (f.name.clone(), f.field_type.to_string(), f.sql_type))
            .collect()
    };
    assert_eq!(describe(&pbf), describe(&json), "{name}: fields");
    assert_eq!(
        pbf.features.len(),
        json.features.len(),
        "{name}: feature count"
    );

    for (i, (a, b)) in pbf.features.iter().zip(&json.features).enumerate() {
        let (a_attrs, b_attrs) = (
            a.attributes.clone().unwrap_or_default(),
            b.attributes.clone().unwrap_or_default(),
        );
        assert_eq!(
            a_attrs.keys().collect::<Vec<_>>(),
            b_attrs.keys().collect::<Vec<_>>()
        );
        for (key, value) in &a_attrs {
            assert!(
                same_value(value, &b_attrs[key]),
                "{name} feature {i} {key}: {value:?} vs {:?}",
                b_attrs[key]
            );
        }

        let (a_parts, b_parts) = (
            a.geometry.as_ref().map(parts),
            b.geometry.as_ref().map(parts),
        );
        let (a_parts, b_parts) = (a_parts.unwrap_or_default(), b_parts.unwrap_or_default());
        assert_eq!(
            a_parts.len(),
            b_parts.len(),
            "{name} feature {i}: part count"
        );
        for (p, (a_part, b_part)) in a_parts.iter().zip(&b_parts).enumerate() {
            assert_eq!(
                a_part.len(),
                b_part.len(),
                "{name} feature {i} part {p}: vertex count"
            );
            for (v, (a_coord, b_coord)) in a_part.iter().zip(b_part).enumerate() {
                for (o, ((x, y), tol)) in a_coord.iter().zip(b_coord).zip(&tolerance).enumerate() {
                    assert!(
                        (x - y).abs() <= *tol,
                        "{name} feature {i} part {p} vertex {v} ordinate {o}: {x} vs {y}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn points_match_json() -> TestResult {
    matches_json::<2>("points")
}

#[test]
fn lines_match_json() -> TestResult {
    matches_json::<2>("lines")
}

#[test]
fn multipart_polygons_match_json() -> TestResult {
    matches_json::<2>("polygons")
}

#[test]
fn z_lines_match_json() -> TestResult {
    matches_json::<3>("lines_z")
}

#[test]
fn m_lines_match_json() -> TestResult {
    matches_json::<3>("lines_m")
}

#[test]
fn zm_lines_match_json() -> TestResult {
    matches_json::<4>("lines_zm")
}

#[test]
fn errors() -> TestResult {
    let collection = FeatureCollectionPBuffer::decode(fixture("lines_zm.pbf")?.as_slice())?;
    assert_eq!(
        FeatureSet::<2>::try_from(collection).err(),
        Some(FromPbfError::Dimensions {
            expected: 4,
            found: 2
        })
    );
    assert_eq!(
        FeatureSet::<2>::try_from(FeatureCollectionPBuffer::default()).err(),
        Some(FromPbfError::NotFeatureResult)
    );
    Ok(())
}

#[test]
fn attributes_without_geometry_match_json() -> TestResult {
    matches_json::<2>("no_geometry")
}

#[test]
fn tables_match_json() -> TestResult {
    matches_json::<2>("table")
}
