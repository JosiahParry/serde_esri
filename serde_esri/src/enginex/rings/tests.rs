use super::*;
use crate::enginex::VertexAttributes;

/// Builds a polygon from rings that do not repeat their first vertex.
fn polygon(rings: &[&[[f64; 2]]]) -> Polygon {
    let mut path_offsets = vec![0];
    for ring in rings {
        path_offsets.push(path_offsets[path_offsets.len() - 1] + ring.len() as i32);
    }
    Polygon::from(MultiPath {
        vertices: VertexAttributes {
            xy: rings.concat(),
            ..Default::default()
        },
        path_offsets,
        path_flags: vec![PathFlag::Closed.into(); rings.len()],
        segments: None,
    })
}

const CLOCKWISE: [[f64; 2]; 4] = [[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [4.0, 0.0]];
const HOLE: [[f64; 2]; 4] = [[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0]];
const CLOCKWISE_EAST: [[f64; 2]; 4] = [[10.0, 0.0], [10.0, 4.0], [14.0, 4.0], [14.0, 0.0]];
const HOLE_EAST: [[f64; 2]; 4] = [[11.0, 1.0], [12.0, 1.0], [12.0, 2.0], [11.0, 2.0]];

#[test]
fn ring_area_is_positive_when_clockwise() {
    let polygon = polygon(&[&CLOCKWISE, &HOLE]);
    assert_eq!(polygon.rings.ring_area(0), Some(16.0));
    assert_eq!(polygon.rings.ring_area(1), Some(-1.0));
    assert_eq!(polygon.rings.ring_area(2), None);
}

#[test]
fn exteriors_start_ogc_polygons() {
    let polygon = polygon(&[&CLOCKWISE, &HOLE, &CLOCKWISE_EAST, &HOLE_EAST]);
    let starts: Vec<_> = polygon
        .rings
        .path_flags
        .iter()
        .map(|f| f.has(PathFlag::OgcStartPolygon))
        .collect();
    assert_eq!(starts, vec![true, false, true, false]);
    assert!(polygon.rings.is_closed_path(3));
    assert_eq!(polygon.ogc_polygons().collect::<Vec<_>>(), vec![0..2, 2..4]);
}

#[test]
fn an_inverted_first_ring_inverts_the_polygon() {
    let counterclockwise: Vec<[f64; 2]> = CLOCKWISE.iter().rev().copied().collect();
    let polygon = polygon(&[&counterclockwise, &CLOCKWISE_EAST]);
    assert_eq!(polygon.ogc_polygons().collect::<Vec<_>>(), vec![0..2]);
}

#[test]
fn unflagged_rings_form_one_polygon() {
    let mut polygon = polygon(&[&CLOCKWISE, &HOLE, &CLOCKWISE_EAST]);
    polygon.rings.path_flags = vec![PathFlag::Closed.into(); 3];
    assert_eq!(polygon.ogc_polygons().collect::<Vec<_>>(), vec![0..3]);
    polygon.update_ogc_flags();
    assert_eq!(polygon.ogc_polygons().collect::<Vec<_>>(), vec![0..2, 2..3]);
}

#[test]
fn reverse_path_keeps_a_closed_path_start() {
    let mut polygon = polygon(&[&CLOCKWISE]);
    polygon.rings.vertices.m = Some(vec![0.0, 1.0, 2.0, 3.0]);
    polygon.rings.reverse_path(0);
    assert_eq!(
        polygon.rings.vertices.xy,
        vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]]
    );
    assert_eq!(polygon.rings.vertices.m, Some(vec![0.0, 3.0, 2.0, 1.0]));
    assert_eq!(polygon.rings.ring_area(0), Some(-16.0));
}
