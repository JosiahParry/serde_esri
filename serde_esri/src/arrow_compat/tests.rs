use super::*;
use crate::arrow_compat::json::FeatureSetJson;
use arrow_array::Array;
use arrow_schema::{DataType, TimeUnit};

fn parse<const N: usize>(json: &str) -> Result<FeatureSet<N>, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

const COUNTIES: &str = r#"{
    "geometryType": "esriGeometryPolygon",
    "spatialReference": {"wkid": 4326},
    "fields": [
        {"name": "OBJECTID", "type": "esriFieldTypeOID"},
        {"name": "NAME", "type": "esriFieldTypeString"},
        {"name": "POP", "type": "esriFieldTypeInteger"},
        {"name": "AREA", "type": "esriFieldTypeDouble"},
        {"name": "UPDATED", "type": "esriFieldTypeDate"},
        {"name": "SHAPE", "type": "esriFieldTypeGeometry"}
    ],
    "features": [
        {
            "attributes": {"OBJECTID": 1, "NAME": "a", "POP": 10, "AREA": 1.5, "UPDATED": 1700000000000},
            "geometry": {"rings": [[[0, 0], [0, 4], [4, 4], [4, 0], [0, 0]]]}
        },
        {"attributes": {"OBJECTID": 2, "NAME": null, "POP": 20}, "geometry": null}
    ]
}"#;

#[test]
fn fields_and_geometry_become_columns() -> Result<(), String> {
    let batch = RecordBatch::try_from(&parse::<2>(COUNTIES)?).map_err(|e| e.to_string())?;
    assert_eq!(batch.num_rows(), 2);

    let schema = batch.schema();
    let names = schema.fields().iter().map(|f| f.name().as_str()).collect::<Vec<_>>();
    assert_eq!(
        names,
        vec!["OBJECTID", "NAME", "POP", "AREA", "UPDATED", "geometry"]
    );
    let types = schema
        .fields()
        .iter()
        .take(5)
        .map(|f| f.data_type().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        types,
        vec![
            DataType::Int64,
            DataType::Utf8,
            DataType::Int32,
            DataType::Float64,
            DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())),
        ]
    );
    assert!(batch.column(1).is_null(1));
    assert!(batch.column(3).is_null(1));

    let geometry = schema.field(5);
    assert_eq!(
        geometry
            .metadata()
            .get("ARROW:extension:name")
            .map(String::as_str),
        Some("geoarrow.multipolygon")
    );
    let extension = geometry.metadata().get("ARROW:extension:metadata");
    assert!(
        extension.is_some_and(|m| m.contains("EPSG:4326")),
        "{extension:?}"
    );
    assert!(batch.column(5).is_null(1));
    Ok(())
}

#[test]
fn attributes_without_geometry() -> Result<(), String> {
    let feature_set = parse::<2>(
        r#"{"fields": [{"name": "N", "type": "esriFieldTypeSmallInteger"}],
            "features": [{"attributes": {"N": 3}}, {"attributes": {"N": 70000}}]}"#,
    )?;
    let batch = RecordBatch::try_from(&feature_set).map_err(|e| e.to_string())?;
    assert_eq!(batch.num_columns(), 1);
    // 70000 does not fit a small integer.
    assert!(batch.column(0).is_valid(0) && batch.column(0).is_null(1));
    Ok(())
}

#[test]
fn esri_wkids_become_esri_codes() -> Result<(), String> {
    let feature_set = parse::<2>(
        r#"{"geometryType": "esriGeometryPoint", "spatialReference": {"wkid": 102100},
            "features": [{"geometry": {"x": 1, "y": 2}}]}"#,
    )?;
    let batch = RecordBatch::try_from(&feature_set).map_err(|e| e.to_string())?;
    let extension = batch
        .schema()
        .field(0)
        .metadata()
        .get("ARROW:extension:metadata")
        .cloned();
    assert!(extension.is_some_and(|m| m.contains("ESRI:102100")));
    Ok(())
}

#[test]
fn errors() -> Result<(), String> {
    let blob =
        parse::<2>(r#"{"fields": [{"name": "B", "type": "esriFieldTypeBlob"}], "features": []}"#)?;
    assert!(matches!(
        RecordBatch::try_from(&blob),
        Err(ToArrowError::UnsupportedField { .. })
    ));

    let mismatched = parse::<2>(
        r#"{"geometryType": "esriGeometryPolygon", "features": [{"geometry": {"x": 1, "y": 2}}]}"#,
    )?;
    assert!(matches!(
        RecordBatch::try_from(&mismatched),
        Err(ToArrowError::GeometryTypeMismatch)
    ));
    Ok(())
}

#[test]
fn feature_set_flags_give_geometries_their_dimension() -> Result<(), String> {
    let feature_set = parse::<3>(
        r#"{"geometryType": "esriGeometryPolyline", "hasM": true,
            "features": [{"geometry": {"paths": [[[0, 0, 5], [1, 1, 6]]]}}]}"#,
    )?;
    let batch = RecordBatch::try_from(&feature_set).map_err(|e| e.to_string())?;
    let streamed = RecordBatch::try_from(FeatureSetJson(
        br#"{"geometryType": "esriGeometryPolyline", "hasM": true,
            "features": [{"geometry": {"paths": [[[0, 0, 5], [1, 1, 6]]]}}]}"#,
    ))
    .map_err(|e| e.to_string())?;
    assert!(format!("{:?}", batch.schema().field(0).data_type()).contains("xym"));
    assert_eq!(batch.schema(), streamed.schema());
    assert_eq!(batch.column(0), streamed.column(0));
    Ok(())
}
