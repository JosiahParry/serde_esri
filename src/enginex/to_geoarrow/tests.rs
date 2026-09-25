use super::*;
use crate::enginex::{MultiPath, PathFlag};
use geo_traits::{
    to_geo::{ToGeoGeometry, ToGeoMultiLineString, ToGeoMultiPoint, ToGeoMultiPolygon, ToGeoPoint},
    CoordTrait, Dimensions, GeometryTrait, MultiPointTrait, PointTrait,
};
use geoarrow_array::{GeoArrowArray, GeoArrowArrayAccessor};

fn multi_path(paths: &[&[[f64; 2]]]) -> MultiPath {
    let mut path_offsets = vec![0];
    for path in paths {
        path_offsets.push(path_offsets[path_offsets.len() - 1] + path.len() as i32);
    }
    MultiPath {
        vertices: VertexAttributes {
            xy: paths.concat(),
            ..Default::default()
        },
        path_offsets,
        path_flags: vec![Default::default(); paths.len()],
        segments: None,
    }
}

const CLOCKWISE: [[f64; 2]; 4] = [[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [4.0, 0.0]];
const HOLE: [[f64; 2]; 4] = [[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0]];
const CLOCKWISE_EAST: [[f64; 2]; 4] = [[10.0, 0.0], [10.0, 4.0], [14.0, 4.0], [14.0, 0.0]];

#[test]
fn multi_polygons_match_their_geo_traits_view() -> Result<(), ToGeoArrowError> {
    let polygons = [
        Some(Polygon::from(multi_path(&[&CLOCKWISE, &HOLE, &CLOCKWISE_EAST]))),
        None,
        Some(Polygon::from(multi_path(&[]))),
    ];
    let array = MultiPolygonArray::try_from(GeometryColumn(&polygons))?;
    assert_eq!(array.len(), 3);
    assert!(array.is_null(1));
    for (i, polygon) in polygons.iter().enumerate() {
        if let Some(polygon) = polygon {
            assert_eq!(array.value(i)?.to_multi_polygon(), polygon.to_multi_polygon());
        }
    }
    Ok(())
}

#[test]
fn multi_line_strings_close_only_closed_paths() -> Result<(), ToGeoArrowError> {
    let mut paths = multi_path(&[&CLOCKWISE, &HOLE]);
    paths.path_flags[1].insert(PathFlag::Closed);
    let polylines = [None, Some(Polyline(paths))];
    let array = MultiLineStringArray::try_from(GeometryColumn(&polylines))?;
    assert!(array.is_null(0));
    let Some(polyline) = &polylines[1] else {
        panic!("expected a polyline");
    };
    assert_eq!(array.value(1)?.to_multi_line_string(), polyline.to_multi_line_string());
    Ok(())
}

#[test]
fn multi_points_keep_z() -> Result<(), ToGeoArrowError> {
    let multi_point = MultiPoint {
        vertices: VertexAttributes {
            xy: vec![[1.0, 2.0], [3.0, 4.0]],
            z: Some(vec![5.0, 6.0]),
            m: None,
            id: Some(vec![7, 8]),
        },
    };
    let array = MultiPointArray::try_from(GeometryColumn(&[Some(multi_point.clone())]))?;
    let value = array.value(0)?;
    assert_eq!(value.dim(), Dimensions::Xyz);
    assert_eq!(value.to_multi_point(), multi_point.to_multi_point());
    let z = value.point(1).and_then(|p| p.coord()).and_then(|c| c.nth(2));
    assert_eq!(z, Some(6.0));
    Ok(())
}

#[test]
fn null_and_empty_points() -> Result<(), ToGeoArrowError> {
    let vertex = Vertex {
        x: 1.0,
        y: 2.0,
        ..Default::default()
    };
    let points = [Some(Point(Some(vertex))), None, Some(Point(None))];
    let array = PointArray::try_from(GeometryColumn(&points))?;
    assert!(!array.is_null(0) && array.is_null(1) && !array.is_null(2));
    assert_eq!(array.value(0)?.to_point(), geo_types::Point::new(1.0, 2.0));
    assert!(array.value(2)?.coord().is_none());
    Ok(())
}

#[test]
fn mixed_dimensions_are_rejected() {
    let flat = MultiPoint {
        vertices: VertexAttributes {
            xy: vec![[0.0, 0.0]],
            ..Default::default()
        },
    };
    let measured = MultiPoint {
        vertices: VertexAttributes {
            m: Some(vec![1.0]),
            ..flat.vertices.clone()
        },
    };
    assert!(matches!(
        MultiPointArray::try_from(GeometryColumn(&[Some(flat), Some(measured)])),
        Err(ToGeoArrowError::MixedDimensions)
    ));
}

#[test]
fn mixed_geometries_build_a_geometry_array() -> Result<(), ToGeoArrowError> {
    let geometries = [
        Some(Geometry::Polygon(Polygon::from(multi_path(&[&CLOCKWISE])))),
        None,
        Some(Geometry::Point(Point(Some(Vertex::default())))),
    ];
    let array = GeometryArray::try_from(GeometryColumn(&geometries))?;
    assert_eq!(array.len(), 3);
    assert!(array.is_null(1));
    for (i, geometry) in geometries.iter().enumerate() {
        if let Some(geometry) = geometry {
            assert_eq!(array.value(i)?.to_geometry(), geometry.to_geometry());
        }
    }
    Ok(())
}
