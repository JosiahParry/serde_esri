use super::*;
use crate::shape::types::{BoundingBox, MultiPartZ, PointM, Range, ZValues};

fn multi_part(parts: Vec<i32>, xy: &[[f64; 2]]) -> MultiPart {
    MultiPart {
        bbox: BoundingBox {
            xmin: 0.0,
            ymin: 0.0,
            xmax: 10.0,
            ymax: 10.0,
        },
        parts,
        points: xy.iter().map(|&[x, y]| shape::types::Point { x, y }).collect(),
    }
}

/// Figure 2 of the specification with Z and M: rings drop their closing vertex, attributes follow.
#[test]
fn polygon_z_drops_closing_vertices() -> Result<(), FromShapeError> {
    let xy = [
        [5.0, 10.0],
        [10.0, 5.0],
        [5.0, 0.0],
        [0.0, 5.0],
        [5.0, 10.0],
        [5.0, 7.0],
        [3.0, 5.0],
        [5.0, 3.0],
        [7.0, 5.0],
        [5.0, 7.0],
    ];
    let z = (0..10).map(f64::from).collect::<Vec<f64>>();
    let shape = Shape::PolygonZ(MultiPartZ {
        xy: multi_part(vec![0, 5], &xy),
        z: ZValues {
            range: Range { min: 0.0, max: 9.0 },
            values: z,
        },
        m: Some(Measures {
            range: Range {
                min: Some(1.0),
                max: Some(1.0),
            },
            values: vec![Some(1.0), None, None, None, None, None, None, None, None, None],
        }),
    });

    let Geometry::Polygon(polygon) = Geometry::try_from(shape)? else {
        panic!("expected a polygon");
    };
    assert_eq!(polygon.ogc_polygons().collect::<Vec<_>>(), vec![0..2]);
    let rings = polygon.rings;
    assert_eq!(rings.path_offsets, vec![0, 4, 8]);
    assert!(rings.is_closed_path(0) && rings.is_closed_path(1));
    assert_eq!(rings.vertices.xy.len(), 8);
    assert_eq!(rings.vertices.xy[4], [5.0, 7.0]);
    assert_eq!(
        rings.vertices.z,
        Some(vec![0.0, 1.0, 2.0, 3.0, 5.0, 6.0, 7.0, 8.0])
    );
    let m = rings.vertices.m.unwrap_or_default();
    assert_eq!(m.first(), Some(&1.0));
    assert!(m[1..].iter().all(|m| m.is_nan()));
    Ok(())
}

#[test]
fn polyline_collapses_empty_parts() -> Result<(), FromShapeError> {
    let shape = Shape::PolyLine(multi_part(
        vec![0, 2, 2, 4],
        &[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]],
    ));
    let Geometry::Polyline(Polyline(path)) = Geometry::try_from(shape)? else {
        panic!("expected a polyline");
    };
    assert_eq!(path.path_offsets, vec![0, 2, 4]);
    assert_eq!(path.vertices.xy.len(), 4);
    assert!(!path.is_closed_path(0));
    Ok(())
}

#[test]
fn no_parts_is_an_empty_path() -> Result<(), FromShapeError> {
    let shape = Shape::PolyLine(multi_part(Vec::new(), &[[0.0, 0.0], [1.0, 1.0]]));
    assert_eq!(
        Geometry::try_from(shape)?,
        Geometry::Polyline(Polyline(MultiPath::default()))
    );
    Ok(())
}

#[test]
fn points() -> Result<(), FromShapeError> {
    let Geometry::Point(Point(Some(vertex))) = Geometry::try_from(Shape::PointM(PointM {
        x: 1.0,
        y: 2.0,
        m: None,
    }))?
    else {
        panic!("expected a point");
    };
    assert_eq!((vertex.x, vertex.y, vertex.z), (1.0, 2.0, None));
    assert!(vertex.m.is_some_and(f64::is_nan));

    let empty = Shape::Point(shape::types::Point {
        x: f64::NAN,
        y: f64::NAN,
    });
    assert_eq!(Geometry::try_from(empty)?, Geometry::Point(Point(None)));
    Ok(())
}

#[test]
fn errors() {
    assert_eq!(
        Geometry::try_from(Shape::Null),
        Err(FromShapeError::NullShape)
    );
    let skips_first_point = Shape::PolyLine(multi_part(vec![1], &[[0.0, 0.0], [1.0, 1.0]]));
    assert_eq!(
        Geometry::try_from(skips_first_point),
        Err(FromShapeError::Corrupted)
    );
    let short_z = Shape::PolyLineZ(MultiPartZ {
        xy: multi_part(vec![0], &[[0.0, 0.0], [1.0, 1.0]]),
        z: ZValues {
            range: Range { min: 0.0, max: 0.0 },
            values: vec![0.0],
        },
        m: None,
    });
    assert_eq!(Geometry::try_from(short_z), Err(FromShapeError::Corrupted));
}
