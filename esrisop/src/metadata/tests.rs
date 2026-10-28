use super::*;
use parquet::{
    file::metadata::KeyValue,
    schema::{parser::parse_message_type, types::SchemaDescriptor},
};
use std::sync::Arc;

const SPEC_XZ: &str = r#"{
  "version": "0.1",
  "type": "xz",
  "wkid": 4326,
  "code": ["geodisplay", "xz-code"],
  "geometryType": "polygon",
  "maxLevel": 20,
  "hasZ": false,
  "hasM": false,
  "fullExtent": {"xmin": -180, "ymin": -90, "xmax": 180, "ymax": 90},
  "encoding": "esriPBF",
  "levels": [
    {
      "column": ["geodisplay", "multiscale-0"],
      "level": 0,
      "resolution": 0.703125,
      "scale": 295829355.4545656,
      "transform": {"scale": [0.703125, 0.703125, 1, 1], "translate": [0, 0, 0, 0]}
    }
  ]
}"#;

const SPEC_Z: &str = r#"{
  "version": "0.1",
  "type": "z",
  "code": ["geodisplay", "zCode"],
  "geometryType": "point",
  "hasZ": false,
  "hasM": false,
  "fullExtent": {"xmin": -180, "ymin": -90, "xmax": 180, "ymax": 90},
  "wkid": 4326,
  "xColumn": ["geodisplay", "x"],
  "yColumn": ["geodisplay", "y"],
  "coordinatePrecision": 16
}"#;

const GEOANALYTICS_SPARK: &str = r#"{"type":"struct","fields":[{"name":"OBJECTID","type":"integer","nullable":false,"metadata":{}},{"name":"geodisplay","type":{"type":"struct","fields":[{"name":"indexkey","type":"long","nullable":false,"metadata":{}},{"name":"xcoord","type":"double","nullable":false,"metadata":{}},{"name":"ycoord","type":"double","nullable":false,"metadata":{}}]},"nullable":true,"metadata":{"coordinatePrecision":20,"writer":{"name":"GeoAnalytics","version":"2026.2.0.246"},"version":"0.1","fullExtent":{"ymin":16.68572908700451,"xmin":-116.33017733600468,"ymax":32.39007437700275,"xmax":-96.1187506390055},"type":"z","yColumn":"ycoord","xColumn":"xcoord","wkid":4326,"code":"indexkey"}}]}"#;

fn file(key_values: &[(&str, &str)]) -> Result<FileMetaData, String> {
    let schema = parse_message_type("message m { required int32 a; }")
        .map(|t| Arc::new(SchemaDescriptor::new(Arc::new(t))))
        .map_err(|e| e.to_string())?;
    let key_values = key_values
        .iter()
        .map(|(k, v)| KeyValue::new(k.to_string(), v.to_string()))
        .collect();
    Ok(FileMetaData::new(
        1,
        0,
        None,
        Some(key_values),
        schema,
        None,
    ))
}

fn child(group: &str, name: &str) -> ColumnPath {
    ColumnPath::Child(group.to_string(), name.to_string())
}

fn parse(key_values: &[(&str, &str)]) -> Result<GeodisplayMetadata, String> {
    let raw = GeodisplayJson::try_from(&file(key_values)?).map_err(|e| e.to_string())?;
    GeodisplayMetadata::try_from(&raw).map_err(|e| e.to_string())
}

#[test]
fn reads_the_spec_xz_example_from_the_root() -> Result<(), String> {
    let GeodisplayMetadata::Xz(xz) = parse(&[(ROOT_KEY, SPEC_XZ)])? else {
        return Err("expected an XZ index".into());
    };
    let level = xz.levels.first().ok_or("no levels")?;
    assert_eq!(xz.code, child("geodisplay", "xz-code"));
    assert_eq!(xz.geometry_type, GeometryType::Polygon);
    assert_eq!(xz.max_level, 20);
    assert_eq!(level.column, child("geodisplay", "multiscale-0"));
    assert_eq!(level.transform.scale, [0.703125, 0.703125, 1.0, 1.0]);
    assert_eq!(xz.display.full_extent.xmin, -180.0);
    assert_eq!(xz.display.wkid, Some(4326));
    Ok(())
}

#[test]
fn reads_the_spec_z_example_from_the_root() -> Result<(), String> {
    let GeodisplayMetadata::Z(z) = parse(&[(ROOT_KEY, SPEC_Z)])? else {
        return Err("expected a Z index".into());
    };
    assert_eq!(z.code, child("geodisplay", "zCode"));
    assert_eq!(z.x_column, child("geodisplay", "x"));
    assert_eq!(z.coordinate_precision, 16);
    Ok(())
}

#[test]
fn reads_geoanalytics_spark_field_metadata() -> Result<(), String> {
    let GeodisplayMetadata::Z(z) = parse(&[(SPARK_KEY, GEOANALYTICS_SPARK)])? else {
        return Err("expected a Z index".into());
    };
    assert_eq!(z.code, child("geodisplay", "indexkey"));
    assert_eq!(z.x_column, child("geodisplay", "xcoord"));
    assert_eq!(z.y_column, child("geodisplay", "ycoord"));
    assert_eq!(z.coordinate_precision, 20);
    assert!(!z.display.has_z);
    assert_eq!(z.display.version, "0.1");
    assert_eq!(
        z.display.writer.map(|w| w.name).as_deref(),
        Some("GeoAnalytics")
    );
    Ok(())
}

#[test]
fn rejects_both_forms_and_neither() -> Result<(), String> {
    let both = file(&[(ROOT_KEY, SPEC_Z), (SPARK_KEY, GEOANALYTICS_SPARK)])?;
    assert!(matches!(
        GeodisplayJson::try_from(&both),
        Err(Error::BothMetadataForms)
    ));
    let spark_without_index = r#"{"type":"struct","fields":[{"name":"a","type":"integer","nullable":false,"metadata":{}}]}"#;
    assert!(matches!(
        GeodisplayJson::try_from(&file(&[(SPARK_KEY, spark_without_index)])?),
        Err(Error::MissingMetadata)
    ));
    Ok(())
}

#[test]
fn picks_the_coarsest_level_detailed_enough() -> Result<(), String> {
    let GeodisplayMetadata::Xz(mut xz) = parse(&[(ROOT_KEY, SPEC_XZ)])? else {
        return Err("expected an XZ index".into());
    };
    let first = xz.levels.first().ok_or("no levels")?.clone();
    xz.levels = (0..3)
        .rev()
        .map(|level| MultiscaleLevel {
            level,
            resolution: first.resolution / 2f64.powi(i32::from(level)),
            column: child("geodisplay", &format!("multiscale-{level}")),
            ..first.clone()
        })
        .collect();
    let level = |resolution: f64| xz.level(resolution).map(|l| l.level).ok();
    assert_eq!(level(10.0), Some(0));
    assert_eq!(level(0.703125), Some(0));
    assert_eq!(level(0.5), Some(1));
    assert_eq!(level(0.01), Some(2));
    assert!(matches!(xz.level(0.0), Err(Error::Resolution(_))));
    Ok(())
}
