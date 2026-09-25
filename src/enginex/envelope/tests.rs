use super::*;
use crate::enginex::{Line, MultiPoint, Point, VertexAttributes};

#[test]
fn covers_every_attribute_and_skips_nan() {
    let multi_point = Geometry::MultiPoint(MultiPoint {
        vertices: VertexAttributes {
            xy: vec![[1.0, 5.0], [-2.0, 3.0], [4.0, 0.0]],
            z: Some(vec![10.0, 30.0, 20.0]),
            m: Some(vec![f64::NAN, 2.0, 1.0]),
            id: Some(vec![7, 3, 9]),
        },
    });
    assert_eq!(
        multi_point.envelope(),
        Envelope {
            xy: Some(Envelope2D {
                xmin: -2.0,
                ymin: 0.0,
                xmax: 4.0,
                ymax: 5.0
            }),
            z: Some(Interval {
                min: 10.0,
                max: 30.0
            }),
            m: Some(Interval { min: 1.0, max: 2.0 }),
            id: Some(Interval { min: 3, max: 9 }),
        }
    );
}

#[test]
fn points_lines_and_empties() {
    let start = Vertex {
        x: 3.0,
        y: 1.0,
        ..Default::default()
    };
    let end = Vertex {
        x: 1.0,
        y: 2.0,
        ..Default::default()
    };
    let line = Geometry::Line(Line { start, end }).envelope();
    assert_eq!(
        line.xy,
        Some(Envelope2D {
            xmin: 1.0,
            ymin: 1.0,
            xmax: 3.0,
            ymax: 2.0
        })
    );
    assert_eq!(line.z, None);

    let point = Geometry::Point(Point(Some(start))).envelope();
    assert_eq!(point.xy.map(|e| (e.xmin, e.xmax)), Some((3.0, 3.0)));
    assert_eq!(Geometry::Point(Point(None)).envelope(), Envelope::default());
}
