use super::*;
use crate::enginex::{Envelope2D, Interval, PathFlag, SegmentType, Segments};

fn points(xy: &[[f64; 2]]) -> Vec<shape::Point> {
    xy.iter().map(|&[x, y]| shape::Point { x, y }).collect()
}

/// Figure 2 of the specification, with Z and M, survives the engine unchanged.
#[test]
fn polygon_z_round_trips() -> Result<(), String> {
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
    let z = [1.0, 2.0, 3.0, 4.0, 1.0, 5.0, 6.0, 7.0, 8.0, 5.0];
    let m = [None, Some(2.0), Some(3.0), None, None, Some(9.0), None, None, None, Some(9.0)];
    let original = Shape::PolygonZ(MultiPartZ {
        xy: MultiPart {
            bbox: BoundingBox {
                xmin: 0.0,
                ymin: 0.0,
                xmax: 10.0,
                ymax: 10.0,
            },
            parts: vec![0, 5],
            points: points(&xy),
        },
        z: ZValues {
            range: Range { min: 1.0, max: 8.0 },
            values: z.to_vec(),
        },
        m: Some(Measures {
            range: Range {
                min: Some(2.0),
                max: Some(9.0),
            },
            values: m.to_vec(),
        }),
    });
    let geometry = Geometry::try_from(original.clone()).map_err(|e| e.to_string())?;
    assert_eq!(Shape::try_from(&geometry).map_err(|e| e.to_string())?, original);
    Ok(())
}

#[test]
fn polylines_close_only_closed_paths() -> Result<(), ToShapeError> {
    let mut path = MultiPath {
        vertices: VertexAttributes {
            xy: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [5.0, 5.0], [6.0, 6.0]],
            ..Default::default()
        },
        path_offsets: vec![0, 3, 5],
        path_flags: vec![Default::default(); 2],
        segments: None,
    };
    path.path_flags[0].insert(PathFlag::Closed);
    let Shape::PolyLine(line) = Shape::try_from(&Geometry::Polyline(Polyline(path)))? else {
        panic!("expected a polyline");
    };
    assert_eq!(line.parts, vec![0, 4]);
    assert_eq!(
        line.points,
        points(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [5.0, 5.0], [6.0, 6.0]])
    );
    Ok(())
}

#[test]
fn envelopes_become_clockwise_rings() -> Result<(), ToShapeError> {
    let envelope = Geometry::Envelope(Envelope {
        xy: Some(Envelope2D {
            xmin: 0.0,
            ymin: 1.0,
            xmax: 2.0,
            ymax: 3.0,
        }),
        z: Some(Interval { min: 4.0, max: 5.0 }),
        m: None,
        id: None,
    });
    let Shape::PolygonZ(polygon) = Shape::try_from(&envelope)? else {
        panic!("expected a PolygonZ");
    };
    assert_eq!(
        polygon.xy.points,
        points(&[[0.0, 1.0], [0.0, 3.0], [2.0, 3.0], [2.0, 1.0], [0.0, 1.0]])
    );
    assert_eq!(polygon.z.values, vec![4.0, 5.0, 4.0, 5.0, 4.0]);
    assert_eq!(polygon.m, None);
    Ok(())
}

#[test]
fn points_map_measures_to_no_data() -> Result<(), ToShapeError> {
    let vertex = Vertex {
        x: 1.0,
        y: 2.0,
        z: None,
        m: Some(f64::NAN),
        id: Some(4),
    };
    assert_eq!(
        Shape::try_from(&Geometry::Point(Point(Some(vertex))))?,
        Shape::PointM(PointM {
            x: 1.0,
            y: 2.0,
            m: None
        })
    );
    Ok(())
}

#[test]
fn curves_are_rejected() {
    let path = MultiPath {
        vertices: VertexAttributes {
            xy: vec![[0.0, 0.0], [1.0, 1.0]],
            ..Default::default()
        },
        path_offsets: vec![0, 2],
        path_flags: vec![Default::default()],
        segments: Some(Segments {
            flags: vec![SegmentType::Bezier.into(), SegmentType::Line.into()],
            param_index: vec![0, -1],
            params: vec![0.0; 6],
        }),
    };
    assert_eq!(
        Shape::try_from(&Geometry::Polyline(Polyline(path))),
        Err(ToShapeError::Curves)
    );
}
