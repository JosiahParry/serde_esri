use super::*;

fn parse<const N: usize>(json: &str) -> Result<EsriGeometry<N>, serde_json::Error> {
    serde_json::from_str(json)
}

#[test]
fn keys_choose_the_variant_in_any_order() -> Result<(), serde_json::Error> {
    assert!(matches!(
        parse::<2>(r#"{"spatialReference": {"wkid": 4326}, "rings": [[[0, 0], [0, 1], [1, 1], [0, 0]]]}"#)?,
        EsriGeometry::Polygon(p) if p.rings.len() == 1 && p.spatialReference.is_some()
    ));
    assert!(matches!(
        parse::<3>(r#"{"hasZ": true, "paths": [[[0, 0, 1], [1, 1, 2]]]}"#)?,
        EsriGeometry::Polyline(p) if p.hasZ == Some(true)
    ));
    assert!(matches!(
        parse::<2>(r#"{"points": [[0, 0], [1, 1]]}"#)?,
        EsriGeometry::MultiPoint(mp) if mp.points.len() == 2
    ));
    assert!(matches!(
        parse::<2>(r#"{"ymax": 3, "xmin": 0, "ymin": 1, "xmax": 2, "zmin": 4}"#)?,
        EsriGeometry::Envelope(e) if e.zmin == Some(4.0) && e.zmax.is_none()
    ));
    assert!(matches!(
        parse::<2>(r#"{"y": 2, "m": 5, "x": 1}"#)?,
        EsriGeometry::Point(p) if p.x == 1.0 && p.m == Some(5.0)
    ));
    Ok(())
}

#[test]
fn serializes_as_before() -> Result<(), serde_json::Error> {
    let json = r#"{"paths":[[[0.0,0.0],[1.0,1.0]]]}"#;
    assert_eq!(serde_json::to_string(&parse::<2>(json)?)?, json);
    Ok(())
}

#[test]
fn unknown_objects_are_rejected() {
    assert!(parse::<2>(r#"{"spatialReference": {"wkid": 4326}}"#).is_err());
    assert!(parse::<2>(r#"{"x": 1}"#).is_err());
}
