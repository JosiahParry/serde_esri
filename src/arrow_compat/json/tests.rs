use super::*;
use crate::features::FeatureSet;
use arrow_array::{Array, Date32Array, Int16Array, Int64Array, TimestampMillisecondArray};

fn stream(json: &str) -> Result<RecordBatch, ToArrowError> {
    RecordBatch::try_from(FeatureSetJson(json.as_bytes()))
}

/// Streaming the JSON matches parsing a `FeatureSet` and converting it through the engine.
fn agree<const N: usize>(json: &str) -> Result<(), String> {
    let feature_set: FeatureSet<N> = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let expected = RecordBatch::try_from(&feature_set).map_err(|e| e.to_string())?;
    let streamed = stream(json).map_err(|e| e.to_string())?;
    assert_eq!(streamed, expected, "{json}");
    Ok(())
}

const POLYGONS: &str = r#"{
    "geometryType": "esriGeometryPolygon",
    "spatialReference": {"wkid": 4326},
    "fields": [
        {"name": "OBJECTID", "type": "esriFieldTypeOID"},
        {"name": "NAME", "type": "esriFieldTypeString"},
        {"name": "POP", "type": "esriFieldTypeInteger"},
        {"name": "AREA", "type": "esriFieldTypeDouble"},
        {"name": "UPDATED", "type": "esriFieldTypeDate"}
    ],
    "features": [
        {
            "attributes": {"OBJECTID": 1, "NAME": "a", "POP": 10, "AREA": 1.5, "UPDATED": 1700000000000},
            "geometry": {"rings": [
                [[0, 0], [0, 4], [4, 4], [4, 0], [0, 0]],
                [[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]],
                [[10, 0], [10, 1], [11, 1], [10, 0]]
            ]}
        },
        {"attributes": {"OBJECTID": 2, "NAME": null, "POP": 20}, "geometry": null},
        {"attributes": {"OBJECTID": 3}, "geometry": {"rings": []}}
    ]
}"#;

#[test]
fn matches_the_feature_set_path() -> Result<(), String> {
    agree::<2>(POLYGONS)?;
    agree::<3>(
        r#"{"geometryType": "esriGeometryPolyline", "hasZ": true, "spatialReference": {"wkid": 102100},
            "features": [
                {"geometry": {"hasZ": true, "paths": [[[0, 0, 1], [1, 1, 2]], [[5, 5, 3], [6, 6, 4]]]}},
                {"geometry": null}
            ]}"#,
    )?;
    agree::<3>(
        r#"{"geometryType": "esriGeometryMultipoint", "hasM": true,
            "features": [{"geometry": {"hasM": true, "points": [[0, 0, 7], [3, 4, 8]]}}]}"#,
    )?;
    agree::<2>(
        r#"{"geometryType": "esriGeometryPoint", "hasZ": true,
            "features": [{"geometry": {"x": 1, "y": 2, "z": 3}}, {"geometry": {"x": 4, "y": 5, "z": 6}}]}"#,
    )
}

#[test]
fn keys_after_features_are_reparsed() -> Result<(), String> {
    let reordered = r#"{
        "features": [{"attributes": {"ID": 7}, "geometry": {"paths": [[[0, 0, 1], [1, 1, 2]]]}}],
        "fields": [{"name": "ID", "type": "esriFieldTypeInteger"}],
        "hasZ": true,
        "geometryType": "esriGeometryPolyline"
    }"#;
    let ordered = r#"{
        "geometryType": "esriGeometryPolyline",
        "hasZ": true,
        "fields": [{"name": "ID", "type": "esriFieldTypeInteger"}],
        "features": [{"attributes": {"ID": 7}, "geometry": {"paths": [[[0, 0, 1], [1, 1, 2]]]}}]
    }"#;
    let (reordered, ordered) = (stream(reordered), stream(ordered));
    let (reordered, ordered) = (
        reordered.map_err(|e| e.to_string())?,
        ordered.map_err(|e| e.to_string())?,
    );
    assert_eq!(reordered, ordered);
    assert_eq!(reordered.num_columns(), 2);
    Ok(())
}

#[test]
fn values_that_do_not_fit_become_null() -> Result<(), String> {
    let batch = stream(
        r#"{"fields": [
                {"name": "SMALL", "type": "esriFieldTypeSmallInteger"},
                {"name": "BIG", "type": "esriFieldTypeBigInteger"},
                {"name": "DAY", "type": "esriFieldTypeDateOnly"},
                {"name": "PHOTO", "type": "esriFieldTypeBlob"}
            ],
            "features": [
                {"attributes": {"SMALL": 70000, "BIG": 9007199254740993, "DAY": "2024-05-01", "PHOTO": "AA=="}},
                {"attributes": {"SMALL": "12", "BIG": 2.5, "DAY": null, "SMALL": 99}},
                {"attributes": {"SMALL": 3.0, "OTHER": [1, 2]}},
                null
            ]}"#,
    )
    .map_err(|e| e.to_string())?;
    assert_eq!(batch.num_rows(), 4);
    assert_eq!(batch.num_columns(), 3, "blob fields are skipped");

    let column = |i: usize| batch.column(i).clone();
    let small = column(0);
    let small = small.as_any().downcast_ref::<Int16Array>().ok_or("SMALL is not Int16")?;
    let small: Vec<_> = small.iter().collect();
    assert_eq!(small, vec![None, Some(12), Some(3), None]);

    let big = column(1);
    let big = big.as_any().downcast_ref::<Int64Array>().ok_or("BIG is not Int64")?;
    assert_eq!(big.iter().collect::<Vec<_>>(), vec![Some(9007199254740993), None, None, None]);

    let day = column(2);
    let day = day.as_any().downcast_ref::<Date32Array>().ok_or("DAY is not Date32")?;
    assert_eq!(day.iter().collect::<Vec<_>>(), vec![Some(19_844), None, None, None]);
    Ok(())
}

/// Every field type from the specification, including the newer date and time types.
#[test]
fn newer_field_types_match_the_feature_set_path() -> Result<(), String> {
    agree::<2>(
        r#"{"fields": [
                {"name": "BIG", "type": "esriFieldTypeBigInteger"},
                {"name": "DAY", "type": "esriFieldTypeDateOnly"},
                {"name": "TIME", "type": "esriFieldTypeTimeOnly"},
                {"name": "STAMP", "type": "esriFieldTypeTimestampOffset"},
                {"name": "GLOBAL", "type": "esriFieldTypeGlobalID"},
                {"name": "DOC", "type": "esriFieldTypeXML"},
                {"name": "SHAPE", "type": "esriFieldTypeGeometry"}
            ],
            "features": [
                {"attributes": {"BIG": 9007199254740993, "DAY": "2003-01-25", "TIME": "21:00:00",
                    "STAMP": "2003-01-25T14:35:00.927-08:00", "GLOBAL": "{A1}", "DOC": "<a/>"}},
                {"attributes": {"BIG": null, "DAY": null, "TIME": "07:30:00.5", "STAMP": null}}
            ]}"#,
    )?;
    let batch = stream(
        r#"{"fields": [{"name": "STAMP", "type": "esriFieldTypeTimestampOffset"}],
            "features": [{"attributes": {"STAMP": "2003-01-25T14:35:00.927-08:00"}}]}"#,
    )
    .map_err(|e| e.to_string())?;
    let stamp = batch.column(0).clone();
    let stamp = stamp
        .as_any()
        .downcast_ref::<TimestampMillisecondArray>()
        .ok_or("STAMP is not a timestamp")?;
    assert_eq!(stamp.value(0), 1_043_534_100_927, "22:35:00.927 UTC");
    Ok(())
}

#[test]
fn geometries_tolerate_odd_coordinates() -> Result<(), String> {
    let batch = stream(
        r#"{"geometryType": "esriGeometryPoint",
            "features": [
                {"geometry": {"x": "NaN", "y": "NaN"}},
                {"attributes": {}},
                {"geometry": {"spatialReference": {"wkid": 4326}, "y": 2, "x": 1, "z": 9}}
            ]}"#,
    )
    .map_err(|e| e.to_string())?;
    let geometry = batch.column(0);
    assert_eq!(geometry.len(), 3);
    assert!(geometry.is_valid(0), "an empty point is not null");
    assert!(geometry.is_null(1), "a missing geometry is null");
    assert!(geometry.is_valid(2));
    Ok(())
}

#[test]
fn errors() {
    assert!(matches!(
        stream(r#"{"error": {"code": 400, "message": "Invalid query", "details": []}}"#),
        Err(ToArrowError::Service { code: Some(400), .. })
    ));
    assert!(matches!(
        stream(r#"{"geometryType": "esriGeometryEnvelope", "features": []}"#),
        Err(ToArrowError::UnsupportedGeometryType(_))
    ));
    assert!(matches!(
        stream(r#"{"features": [{"attributes": {]}"#),
        Err(ToArrowError::Json(_))
    ));
}

#[test]
fn the_latest_wkid_sets_the_crs() -> Result<(), String> {
    let batch = stream(
        r#"{"geometryType": "esriGeometryPoint", "spatialReference": {"wkid": 102100, "latestWkid": 3857},
            "features": [{"geometry": {"x": 1, "y": 2}}]}"#,
    )
    .map_err(|e| e.to_string())?;
    let extension = batch.schema().field(0).metadata().get("ARROW:extension:metadata").cloned();
    assert!(extension.is_some_and(|m| m.contains("EPSG:3857")));
    Ok(())
}
