use super::*;

fn parse<const N: usize>(json: &str) -> Result<EsriGeometry<N>, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

#[test]
fn polygon_rings_drop_their_closing_vertex() -> Result<(), String> {
    let polygon = parse::<3>(
        r#"{"hasZ": true, "rings": [
            [[0, 0, 1], [0, 4, 2], [4, 4, 3], [4, 0, 4], [0, 0, 1]],
            [[1, 1, 5], [2, 1, 6], [2, 2, 7], [1, 1, 5]]
        ]}"#,
    )?;
    let Geometry::Polygon(polygon) = Geometry::try_from(&polygon).map_err(|e| e.to_string())? else {
        return Err("expected a polygon".into());
    };
    assert_eq!(polygon.rings.path_offsets, vec![0, 4, 7]);
    assert_eq!(
        polygon.rings.vertices.z,
        Some(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0])
    );
    assert_eq!(polygon.rings.vertices.m, None);
    assert_eq!(polygon.ogc_polygons().collect::<Vec<_>>(), vec![0..2]);
    Ok(())
}

#[test]
fn m_sits_after_z_or_in_its_place() -> Result<(), String> {
    let measured =
        parse::<3>(r#"{"hasM": true, "paths": [[[0, 0, 7], [1, 1, 8]]]}"#)?;
    let Geometry::Polyline(Polyline(path)) =
        Geometry::try_from(&measured).map_err(|e| e.to_string())?
    else {
        return Err("expected a polyline".into());
    };
    assert_eq!(path.vertices.m, Some(vec![7.0, 8.0]));
    assert_eq!(path.vertices.z, None);

    let both =
        parse::<4>(r#"{"hasZ": true, "hasM": true, "points": [[0, 0, 1, 2]]}"#)?;
    let Geometry::MultiPoint(mp) = Geometry::try_from(&both).map_err(|e| e.to_string())? else {
        return Err("expected a multipoint".into());
    };
    assert_eq!((mp.vertices.z, mp.vertices.m), (Some(vec![1.0]), Some(vec![2.0])));
    Ok(())
}

#[test]
fn points_and_envelopes() -> Result<(), String> {
    let point = parse::<2>(r#"{"x": 1, "y": 2, "m": 3}"#)?;
    assert_eq!(
        Geometry::try_from(&point).map_err(|e| e.to_string())?,
        Geometry::Point(Point(Some(Vertex {
            x: 1.0,
            y: 2.0,
            z: None,
            m: Some(3.0),
            id: None
        })))
    );

    let envelope =
        parse::<2>(r#"{"xmin": 0, "ymin": 1, "xmax": 2, "ymax": 3, "zmin": 4, "zmax": 5}"#)?;
    let Geometry::Envelope(envelope) = Geometry::try_from(&envelope).map_err(|e| e.to_string())?
    else {
        return Err("expected an envelope".into());
    };
    assert_eq!(envelope.z, Some(Interval { min: 4.0, max: 5.0 }));
    Ok(())
}

#[test]
fn flags_must_match_the_coordinate_width() -> Result<(), String> {
    let mismatched = parse::<3>(r#"{"hasZ": true, "hasM": true, "points": [[0, 0, 1]]}"#)?;
    assert_eq!(
        Geometry::try_from(&mismatched),
        Err(FromEsriError::Dimensions {
            expected: 4,
            found: 3
        })
    );
    Ok(())
}
