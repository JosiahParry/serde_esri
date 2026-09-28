use super::*;
use geo_types::{coord, line_string, point, polygon, Coord, LineString, MultiPolygon, Rect};

fn xy(path: &MultiPath, index: usize) -> Vec<[f64; 2]> {
    let range = path.path_range(index).unwrap_or(0..0);
    path.vertices.xy[range].to_vec()
}

/// A GeoJSON-style polygon: counterclockwise exterior, clockwise hole, closed rings.
#[test]
fn rings_are_reoriented_and_opened_like_the_engine() -> Result<(), FromGeoTraitsError> {
    let polygon = polygon!(
        exterior: [(x: 0.0, y: 0.0), (x: 4.0, y: 0.0), (x: 4.0, y: 4.0), (x: 0.0, y: 4.0), (x: 0.0, y: 0.0)],
        interiors: [[(x: 1.0, y: 1.0), (x: 1.0, y: 2.0), (x: 2.0, y: 2.0), (x: 2.0, y: 1.0), (x: 1.0, y: 1.0)]],
    );
    let Geometry::Polygon(converted) = Geometry::from_geo_traits(&polygon)? else {
        panic!("expected a polygon");
    };
    let rings = &converted.rings;
    // Reversed keeping the first vertex, as reversePath does for closed paths.
    assert_eq!(xy(rings, 0), vec![[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [4.0, 0.0]]);
    assert_eq!(xy(rings, 1), vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0]]);
    assert!(rings.ring_area(0).is_some_and(|a| a > 0.0));
    assert!(rings.ring_area(1).is_some_and(|a| a < 0.0));
    assert_eq!(converted.ogc_polygons().collect::<Vec<_>>(), vec![0..2]);
    Ok(())
}

#[test]
fn multi_polygons_start_an_ogc_polygon_per_exterior() -> Result<(), FromGeoTraitsError> {
    let square = |x0: f64| {
        polygon![(x: x0, y: 0.0), (x: x0, y: 1.0), (x: x0 + 1.0, y: 1.0), (x: x0 + 1.0, y: 0.0)]
    };
    let multi_polygon = MultiPolygon::new(vec![square(0.0), square(5.0)]);
    let Geometry::Polygon(converted) = Geometry::from_geo_traits(&multi_polygon)? else {
        panic!("expected a polygon");
    };
    assert_eq!(converted.ogc_polygons().collect::<Vec<_>>(), vec![0..1, 1..2]);
    Ok(())
}

#[test]
fn engine_polygons_round_trip_through_geo_traits() -> Result<(), FromGeoTraitsError> {
    let polygon = Polygon::from(MultiPath {
        vertices: VertexAttributes {
            xy: vec![
                [0.0, 0.0],
                [0.0, 4.0],
                [4.0, 4.0],
                [4.0, 0.0],
                [1.0, 1.0],
                [2.0, 1.0],
                [2.0, 2.0],
                [10.0, 0.0],
                [10.0, 1.0],
                [11.0, 1.0],
            ],
            z: Some((0..10).map(f64::from).collect()),
            ..Default::default()
        },
        path_offsets: vec![0, 4, 7, 10],
        path_flags: vec![PathFlag::Closed.into(); 3],
        segments: None,
    });
    let geometry = Geometry::Polygon(polygon);
    assert_eq!(Geometry::from_geo_traits(&geometry)?, geometry);
    Ok(())
}

#[test]
fn lines_points_and_rects() -> Result<(), FromGeoTraitsError> {
    let line: LineString = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0)];
    let Geometry::Polyline(Polyline(path)) = Geometry::from_geo_traits(&line)? else {
        panic!("expected a polyline");
    };
    assert_eq!(path.path_offsets, vec![0, 2]);
    assert!(!path.is_closed_path(0));

    let converted = Geometry::from_geo_traits(&point!(x: 1.0, y: 2.0))?;
    assert_eq!(
        converted,
        Geometry::Point(Point(Some(Vertex {
            x: 1.0,
            y: 2.0,
            ..Default::default()
        })))
    );

    let rect = Rect::new(coord! { x: 0.0, y: 1.0 }, Coord { x: 2.0, y: 3.0 });
    let Geometry::Envelope(envelope) = Geometry::from_geo_traits(&rect)? else {
        panic!("expected an envelope");
    };
    assert_eq!(envelope.xy.map(|e| (e.xmin, e.ymax)), Some((0.0, 3.0)));
    Ok(())
}

#[test]
fn geometry_collections_are_rejected() {
    let collection = geo_types::GeometryCollection::<f64>::new_from(Vec::new());
    assert_eq!(
        Geometry::from_geo_traits(&collection),
        Err(FromGeoTraitsError::GeometryCollection)
    );
}
