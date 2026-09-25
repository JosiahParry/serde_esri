use super::*;
use crate::{
    enginex::{Envelope2D, Interval, PathFlag},
    shape::{self, MultiPart, Shape},
};
use geo_traits::to_geo::{ToGeoGeometry, ToGeoMultiLineString, ToGeoMultiPolygon, ToGeoPoint};
use geo_types::{coord, Coord, LineString, MultiLineString, MultiPolygon};

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

fn line_string(xy: &[[f64; 2]]) -> LineString {
    xy.iter().map(|&[x, y]| Coord { x, y }).collect()
}

const CLOCKWISE: [[f64; 2]; 4] = [[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [4.0, 0.0]];
const HOLE: [[f64; 2]; 4] = [[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0]];
const CLOCKWISE_EAST: [[f64; 2]; 4] = [[10.0, 0.0], [10.0, 4.0], [14.0, 4.0], [14.0, 0.0]];

#[test]
fn polygon_is_a_multi_polygon_of_closed_rings() {
    let polygon = Polygon::from(multi_path(&[&CLOCKWISE, &HOLE, &CLOCKWISE_EAST]));
    assert_eq!(polygon.num_polygons(), 2);
    assert_eq!(polygon.dim(), Dimensions::Xy);

    let closed = |ring: &[[f64; 2]]| line_string(&[ring, &ring[..1]].concat());
    let expected = MultiPolygon::new(vec![
        geo_types::Polygon::new(closed(&CLOCKWISE), vec![closed(&HOLE)]),
        geo_types::Polygon::new(closed(&CLOCKWISE_EAST), Vec::new()),
    ]);
    assert_eq!(polygon.to_multi_polygon(), expected);
}

#[test]
fn polyline_closes_only_closed_paths() {
    let mut path = multi_path(&[&CLOCKWISE, &HOLE]);
    path.path_flags[1].insert(PathFlag::Closed);
    let expected = MultiLineString::new(vec![
        line_string(&CLOCKWISE),
        line_string(&[&HOLE[..], &HOLE[..1]].concat()),
    ]);
    assert_eq!(Polyline(path).to_multi_line_string(), expected);
}

#[test]
fn vertex_ordinates_follow_dimensions() {
    let xyzm = Vertex {
        x: 1.0,
        y: 2.0,
        z: Some(3.0),
        m: Some(4.0),
        id: Some(7),
    };
    assert_eq!(xyzm.dim(), Dimensions::Xyzm);
    assert_eq!((xyzm.nth(2), xyzm.nth(3), xyzm.nth(4)), (Some(3.0), Some(4.0), None));

    let xym = Vertex { z: None, ..xyzm };
    assert_eq!(xym.dim(), Dimensions::Xym);
    assert_eq!((xym.nth(2), xym.nth(3)), (Some(4.0), None));

    assert_eq!(Point(Some(xyzm)).to_point(), geo_types::Point::new(1.0, 2.0));
    assert!(Point(None).coord().is_none());
}

#[test]
fn geometry_dispatches_by_kind() {
    let envelope = Geometry::Envelope(Envelope {
        xy: Some(Envelope2D {
            xmin: 0.0,
            ymin: 1.0,
            xmax: 2.0,
            ymax: 3.0,
        }),
        z: Some(Interval { min: 5.0, max: 6.0 }),
        m: None,
        id: None,
    });
    assert_eq!(envelope.dim(), Dimensions::Xyz);
    assert_eq!(
        envelope.to_geometry(),
        geo_types::Geometry::Rect(geo_types::Rect::new(
            coord! { x: 0.0, y: 1.0 },
            coord! { x: 2.0, y: 3.0 }
        ))
    );
    let polygon = Geometry::Polygon(Polygon::from(multi_path(&[&CLOCKWISE])));
    assert!(matches!(polygon.as_type(), GeometryType::MultiPolygon(_)));
}

/// A shapefile polygon read into the engine exposes the same rings, closed again.
#[test]
fn shapefile_polygon_round_trips_through_the_engine() -> Result<(), crate::enginex::FromShapeError> {
    let ring = [[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [4.0, 0.0], [0.0, 0.0]];
    let shape = Shape::Polygon(MultiPart {
        bbox: shape::BoundingBox {
            xmin: 0.0,
            ymin: 0.0,
            xmax: 4.0,
            ymax: 4.0,
        },
        parts: vec![0],
        points: ring.iter().map(|&[x, y]| shape::Point { x, y }).collect(),
    });
    let geometry = Geometry::try_from(shape)?;
    assert_eq!(
        geometry.to_geometry(),
        geo_types::Geometry::MultiPolygon(MultiPolygon::new(vec![geo_types::Polygon::new(
            line_string(&ring),
            Vec::new()
        )]))
    );
    Ok(())
}
