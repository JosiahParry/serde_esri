use super::*;
use crate::features::Feature;

#[test]
fn values_round_trip() -> Result<(), serde_json::Error> {
    let json = r#"{"OWNER":"Joe Smith","VALUE":94820.37,"APPROVED":true,"LASTUPDATE":1227663551096,"NOTE":null}"#;
    let feature: Feature<2> = serde_json::from_str(&format!(r#"{{"attributes":{json}}}"#))?;
    let attributes = feature.attributes.unwrap_or_default();
    let keys: Vec<_> = attributes.keys().map(String::as_str).collect();
    assert_eq!(keys, ["OWNER", "VALUE", "APPROVED", "LASTUPDATE", "NOTE"], "field order is kept");
    assert_eq!(attributes["LASTUPDATE"], EsriValue::Int(1_227_663_551_096));
    assert_eq!(attributes["VALUE"].as_f64(), Some(94820.37));
    assert!(attributes["NOTE"].is_null());
    assert_eq!(serde_json::to_string(&attributes)?, json);
    Ok(())
}

#[test]
fn large_and_unexpected_values() -> Result<(), serde_json::Error> {
    let value: EsriValue = serde_json::from_str("9007199254740993")?;
    assert_eq!(value, EsriValue::Int(9_007_199_254_740_993));
    let value: EsriValue = serde_json::from_str("18446744073709551615")?;
    assert_eq!(value, EsriValue::Float(18_446_744_073_709_551_615.0));
    let value: EsriValue = serde_json::from_str(r#"[1, {"a": 2}]"#)?;
    assert!(value.is_null());
    Ok(())
}

#[test]
fn builds_from_rust_values() {
    assert_eq!(EsriValue::from(Some(3)), EsriValue::Int(3));
    assert_eq!(EsriValue::from(None::<&str>), EsriValue::Null);
    assert_eq!(EsriValue::from("a"), EsriValue::String("a".into()));
}
