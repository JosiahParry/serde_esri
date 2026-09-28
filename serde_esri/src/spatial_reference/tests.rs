use super::*;

#[test]
fn keys_are_camel_case() -> Result<(), serde_json::Error> {
    let json = r#"{"wkid":102100,"latestWkid":3857,"vcsWkid":5703,"latestVcsWkid":5703}"#;
    let sr: SpatialReference = serde_json::from_str(json)?;
    assert_eq!(sr.latest_wkid, Some(3857));
    assert_eq!(sr.latest_vcs_wkid, Some(5703));
    assert_eq!(serde_json::to_string(&sr)?, json);
    Ok(())
}
