use super::*;

#[test]
fn vertex_description_bits_match_java() {
    let d: VertexDescription = [Attribute::Z, Attribute::Id].into_iter().collect();
    assert_eq!(u16::from(d), 0b1011);
    assert!(d.has(Attribute::Position));
    assert!(!d.has(Attribute::M));
    assert_eq!(VertexDescription::try_from(0b1011), Ok(d));
    assert_eq!(VertexDescription::try_from(0b1_0001), Err(0b1_0001));
    assert_eq!(VertexDescription::try_from(0b0010), Err(0b0010));
}

#[test]
fn segment_flags_decode() {
    let mut f = SegmentFlags::from(SegmentType::Bezier);
    assert_eq!(f.segment_type(), Some(SegmentType::Bezier));
    f.mark_densified();
    assert_eq!(f.0, 10);
    assert!(f.is_densified());
    assert_eq!(f.segment_type(), Some(SegmentType::Bezier));
    assert_eq!(SegmentFlags(3).segment_type(), None);
}

#[test]
fn multipath_paths_and_segments() {
    let mut closed = PathFlags::from(PathFlag::Closed);
    closed.insert(PathFlag::HasNonlinearSegments);
    let segments = Segments {
        flags: vec![
            SegmentType::Line.into(),
            SegmentType::Bezier.into(),
            SegmentType::Line.into(),
            SegmentType::Line.into(),
            SegmentType::Line.into(),
        ],
        param_index: vec![-1, 0, -1, -1, -1],
        params: vec![1.2, 0.3, 0.0, 1.1, 0.7, 0.0],
    };
    assert_eq!(segments.params(0), None);
    assert_eq!(
        segments.bezier_control_points(1),
        Some([[1.2, 0.3, 0.0], [1.1, 0.7, 0.0]])
    );

    let path = MultiPath {
        vertices: VertexAttributes {
            xy: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [5.0, 5.0], [6.0, 6.0]],
            m: Some(vec![0.0, 1.0, 2.0, 3.0, 4.0]),
            ..Default::default()
        },
        path_offsets: vec![0, 3, 5],
        path_flags: vec![closed, PathFlags::default()],
        segments: Some(segments),
    };

    assert_eq!(path.path_count(), 2);
    assert_eq!(path.path_range(1), Some(3..5));
    assert!(path.is_closed_path(0));
    assert!(!path.is_closed_path(1));
    assert!(path.has_nonlinear_segments());
    assert_eq!(path.segment_type(1), Some(SegmentType::Bezier));
    assert_eq!(u16::from(path.vertices.description()), 0b101);
    assert_eq!(path.vertices.get(4).and_then(|v| v.m), Some(4.0));
}

#[test]
fn geometry_type_codes_match_java() {
    assert_eq!(GeometryType::Polygon as i32, 1736);
    assert_eq!(GeometryType::Polyline as i32, 1607);
    assert_eq!(GeometryType::Point as i32, 33);
}
