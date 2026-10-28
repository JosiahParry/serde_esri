use super::*;
use crate::enginex::{
    flags::{PathFlag, SegmentType},
    geometry::{MultiPoint, Polygon, Segments},
};

fn read(buffer: &EsriShapeBuffer) -> Result<Option<Geometry>, ShapeError> {
    buffer.as_shape().try_into()
}

fn type_code(buffer: &EsriShapeBuffer) -> Option<u32> {
    let bytes = buffer.0.first_chunk::<4>()?;
    Some(u32::from_le_bytes(*bytes))
}

/// Rings without their closing vertex, with Z, M, and IDs.
fn polygon() -> Polygon {
    Polygon::from(MultiPath {
        vertices: VertexAttributes {
            xy: vec![[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [4.0, 0.0], [1.0, 1.0], [2.0, 1.0], [2.0, 2.0]],
            z: Some(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
            m: Some(vec![0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5]),
            id: Some(vec![10, 11, 12, 13, 14, 15, 16]),
        },
        path_offsets: vec![0, 4, 7],
        path_flags: vec![PathFlag::Closed.into(); 2],
        segments: None,
    })
}

#[test]
fn polygons_with_ids_round_trip_as_general_types() -> Result<(), ShapeError> {
    let geometry = Geometry::Polygon(polygon());
    let buffer = EsriShapeBuffer::try_from(&geometry)?;
    assert_eq!(
        type_code(&buffer),
        Some(ShapeType::GeneralPolygon.code() | HAS_ZS | HAS_MS | HAS_IDS)
    );
    assert_eq!(read(&buffer)?, Some(geometry));
    Ok(())
}

#[test]
fn basic_types_follow_z_and_m() -> Result<(), ShapeError> {
    let point_z = Geometry::Point(Point(Some(Vertex {
        x: 1.0,
        y: 2.0,
        z: Some(3.0),
        ..Default::default()
    })));
    let buffer = EsriShapeBuffer::try_from(&point_z)?;
    assert_eq!(type_code(&buffer), Some(ShapeType::PointZ.code()));
    assert_eq!(read(&buffer)?, Some(point_z));

    let multi_point_m = Geometry::MultiPoint(MultiPoint {
        vertices: VertexAttributes {
            xy: vec![[1.0, 2.0], [3.0, 4.0]],
            m: Some(vec![5.0, 6.0]),
            ..Default::default()
        },
    });
    let buffer = EsriShapeBuffer::try_from(&multi_point_m)?;
    assert_eq!(type_code(&buffer), Some(ShapeType::MultiPointM.code()));
    assert_eq!(read(&buffer)?, Some(multi_point_m));
    Ok(())
}

#[test]
fn closed_polyline_paths_gain_a_closing_vertex() -> Result<(), ShapeError> {
    let mut path = MultiPath {
        vertices: VertexAttributes {
            xy: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            ..Default::default()
        },
        path_offsets: vec![0, 3],
        path_flags: vec![Default::default()],
        segments: None,
    };
    path.path_flags[0].insert(PathFlag::Closed);
    let buffer = EsriShapeBuffer::try_from(&Geometry::Polyline(Polyline(path)))?;
    let Some(Geometry::Polyline(Polyline(read_back))) = read(&buffer)? else {
        panic!("expected a polyline");
    };
    assert_eq!(
        read_back.vertices.xy,
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]
    );
    Ok(())
}

#[test]
fn null_and_empty_shapes() -> Result<(), ShapeError> {
    assert_eq!(read(&EsriShapeBuffer::default())?, None);
    let empty = Geometry::Point(Point(None));
    assert_eq!(read(&EsriShapeBuffer::try_from(&empty)?)?, Some(empty));
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
        EsriShapeBuffer::try_from(&Geometry::Polyline(Polyline(path))),
        Err(ShapeError::UnsupportedCurves)
    );
}
