use super::*;
use crate::enginex::geometry::Geometry;
use geo_traits::{to_geo::ToGeoMultiPolygon, CoordTrait};

fn parse<const N: usize>(json: &str) -> Result<EsriGeometry<N>, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Two clockwise exteriors, the first with a counterclockwise hole.
const MULTI_POLYGON: &str = r#"{"rings": [
    [[0, 0], [0, 4], [4, 4], [4, 0], [0, 0]],
    [[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]],
    [[10, 0], [10, 1], [11, 1], [10, 0]]
]}"#;

#[test]
fn polygons_group_rings_by_orientation() -> Result<(), String> {
    let EsriGeometry::Polygon(polygon) = parse::<2>(MULTI_POLYGON)? else {
        return Err("expected a polygon".into());
    };
    assert_eq!(polygon.num_polygons(), 2);
    let multi_polygon = polygon.to_multi_polygon();
    assert_eq!(multi_polygon.0[0].interiors().len(), 1);
    assert_eq!(multi_polygon.0[1].interiors().len(), 0);
    assert_eq!(multi_polygon.0[1].exterior().0.len(), 4);
    Ok(())
}

#[test]
fn measures_follow_the_flags() -> Result<(), String> {
    let EsriGeometry::Polyline(polyline) =
        parse::<3>(r#"{"hasM": true, "paths": [[[0, 0, 7], [1, 1, 8]]]}"#)?
    else {
        return Err("expected a polyline".into());
    };
    assert_eq!(polyline.dim(), Dimensions::Xym);
    let m = polyline
        .line_string(0)
        .and_then(|line| line.coord(1))
        .and_then(|c| c.nth(2));
    assert_eq!(m, Some(8.0));
    Ok(())
}

/// Reading through geo-traits into the engine matches converting directly.
#[test]
fn geo_traits_and_direct_conversion_agree() -> Result<(), String> {
    fn agree<const N: usize>(json: &str) -> Result<(), String> {
        let esri = parse::<N>(json)?;
        let direct = Geometry::try_from(&esri).map_err(|e| e.to_string())?;
        let traits = Geometry::from_geo_traits(&esri).map_err(|e| e.to_string())?;
        assert_eq!(direct, traits, "{json}");
        Ok(())
    }
    agree::<2>(MULTI_POLYGON)?;
    agree::<3>(r#"{"hasZ": true, "paths": [[[0, 0, 1], [1, 1, 2]], [[5, 5, 3], [6, 6, 4]]]}"#)?;
    agree::<2>(r#"{"points": [[0, 0], [3, 4]]}"#)?;
    agree::<2>(r#"{"x": 1, "y": 2, "z": 3}"#)
}
