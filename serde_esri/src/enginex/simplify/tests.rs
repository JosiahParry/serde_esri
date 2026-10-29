use super::*;
use crate::enginex::{
    flags::{PathFlag, PathFlags, SegmentType},
    geometry::Segments,
};

const OPEN: PathFlags = PathFlags(0);
const CLOSED: PathFlags = PathFlags(PathFlag::Closed as u8);
const SQUARE: [[f64; 2]; 4] = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];

fn multi_path(paths: &[(&[[f64; 2]], PathFlags)]) -> MultiPath {
    let ends = paths.iter().scan(0, |end, (xy, _)| {
        *end += xy.len() as i32;
        Some(*end)
    });
    MultiPath {
        vertices: VertexAttributes {
            xy: paths
                .iter()
                .flat_map(|(xy, _)| xy.iter().copied())
                .collect(),
            ..Default::default()
        },
        path_offsets: std::iter::once(0).chain(ends).collect(),
        path_flags: paths.iter().map(|&(_, flags)| flags).collect(),
        segments: None,
    }
}

fn simplified(xy: &[[f64; 2]], flags: PathFlags, tolerance: f64) -> Option<Vec<[f64; 2]>> {
    multi_path(&[(xy, flags)])
        .simplify(tolerance)
        .map(|paths| paths.vertices.xy)
}

#[test]
fn keeps_paths_of_at_most_two_vertices() {
    let xy = [[0.0, 0.0], [2.0, 0.0]];
    for count in 0..=2 {
        assert_eq!(
            simplified(&xy[..count], OPEN, 1.0),
            Some(xy[..count].to_vec())
        );
    }
}

#[test]
fn drops_collinear_vertices_at_zero_tolerance() {
    let xy = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
    assert_eq!(
        simplified(&xy, OPEN, 0.0),
        Some(vec![[0.0, 0.0], [3.0, 0.0]])
    );
}

#[test]
fn drops_vertices_at_the_tolerance_and_keeps_those_beyond_it() {
    assert_eq!(
        simplified(&[[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]], OPEN, 1.0),
        Some(vec![[0.0, 0.0], [2.0, 0.0]])
    );
    let bend = [[0.0, 0.0], [1.0, 1.01], [2.0, 0.0]];
    assert_eq!(simplified(&bend, OPEN, 1.0), Some(bend.to_vec()));
    let bend = [[0.0, 0.0], [1.0, 0.01], [2.0, 0.0]];
    assert_eq!(simplified(&bend, OPEN, 0.0), Some(bend.to_vec()));
}

#[test]
fn measures_xy_only_and_keeps_every_attribute_of_the_kept_vertices() {
    let paths = MultiPath {
        vertices: VertexAttributes {
            xy: vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [2.0, 2.0]],
            z: Some(vec![1.0, 1000.0, 3.0, 5.0]),
            m: Some(vec![2.0, -1000.0, 4.0, 6.0]),
            id: Some(vec![10, 11, 12, 13]),
        },
        path_offsets: vec![0, 4],
        path_flags: vec![OPEN],
        segments: None,
    };
    let expected = VertexAttributes {
        xy: vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]],
        z: Some(vec![1.0, 3.0, 5.0]),
        m: Some(vec![2.0, 4.0, 6.0]),
        id: Some(vec![10, 12, 13]),
    };
    assert_eq!(
        paths.simplify(0.1).map(|paths| paths.vertices),
        Some(expected)
    );
}

#[test]
fn keeps_every_bend_beyond_the_tolerance_in_source_order() {
    let zigzag = [[0.0, 0.0], [1.0, 3.0], [2.0, -3.0], [3.0, 3.0], [4.0, 0.0]];
    assert_eq!(simplified(&zigzag, OPEN, 0.1), Some(zigzag.to_vec()));
}

#[test]
fn measures_distance_to_the_segment_not_the_line() {
    let beyond = [[0.0, 0.0], [2.0, 0.0], [1.0, 0.0]];
    assert_eq!(simplified(&beyond, OPEN, 0.5), Some(beyond.to_vec()));
    let before = [[0.0, 0.0], [-2.0, 1.0], [1.0, 0.0]];
    assert_eq!(simplified(&before, OPEN, 2.2), Some(before.to_vec()));
    assert_eq!(
        simplified(&before, OPEN, 2.3),
        Some(vec![[0.0, 0.0], [1.0, 0.0]])
    );
}

#[test]
fn keeps_removed_vertices_within_the_tolerance_of_their_segment() {
    let xy = (0..128)
        .map(|i| [f64::from(i), f64::from(i % 7)])
        .collect::<Vec<_>>();
    let tolerance = 1.5;
    let kept = simplified(&xy, OPEN, tolerance).unwrap_or_default();
    assert!(kept.len() < xy.len());
    assert_eq!(kept.first(), xy.first());
    assert_eq!(kept.last(), xy.last());
    for pair in kept.windows(2) {
        let ([ax, ay], [bx, by]) = (pair[0], pair[1]);
        let (dx, dy) = (bx - ax, by - ay);
        for &[x, y] in xy.iter().filter(|&&[x, _]| x > ax && x < bx) {
            let along = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            let (ox, oy) = (x - ax - along * dx, y - ay - along * dy);
            assert!(ox * ox + oy * oy <= tolerance * tolerance, "[{x}, {y}]");
        }
    }
}

#[test]
fn simplifies_ten_thousand_collinear_vertices() {
    let xy = (0..10_000).map(|i| [f64::from(i), 0.0]).collect::<Vec<_>>();
    assert_eq!(
        simplified(&xy, OPEN, 1.0),
        Some(vec![[0.0, 0.0], [9999.0, 0.0]])
    );
}

#[test]
fn keeps_a_triangle_of_a_ring_at_any_tolerance() {
    let triangle = [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]];
    assert_eq!(
        simplified(&triangle, CLOSED, 100.0),
        Some(triangle.to_vec())
    );
    assert_eq!(
        simplified(&SQUARE, CLOSED, 100.0),
        Some(SQUARE[..3].to_vec())
    );
}

#[test]
fn keeps_every_ring_vertex_at_zero_tolerance() {
    assert_eq!(simplified(&SQUARE, CLOSED, 0.0), Some(SQUARE.to_vec()));
}

#[test]
fn treats_a_path_that_returns_to_its_start_as_a_ring() {
    let ring = [SQUARE.as_slice(), &[[0.0, 0.0]]].concat();
    assert_eq!(
        simplified(&ring, OPEN, 100.0),
        Some(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]])
    );
}

#[test]
fn simplifies_each_path_on_its_own() {
    let line = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
    let ring = [
        [10.0, 0.0],
        [12.0, 0.01],
        [14.0, 0.0],
        [14.0, 4.0],
        [10.0, 4.0],
    ];
    let expected = multi_path(&[
        (&[[0.0, 0.0], [2.0, 0.0]], OPEN),
        (
            &[[10.0, 0.0], [14.0, 0.0], [14.0, 4.0], [10.0, 4.0]],
            CLOSED,
        ),
    ]);
    assert_eq!(
        multi_path(&[(&line, OPEN), (&ring, CLOSED)]).simplify(0.5),
        Some(expected)
    );
}

#[test]
fn returns_none_for_curves() {
    let paths = MultiPath {
        segments: Some(Segments {
            flags: vec![
                SegmentType::Bezier.into(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            param_index: vec![0, -1, -1, -1],
            params: vec![0.0; 6],
        }),
        ..multi_path(&[(&SQUARE, OPEN)])
    };
    assert_eq!(paths.simplify(1.0), None);
}
