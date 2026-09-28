use super::*;

#[test]
fn null() {
    assert_eq!(Buf::default().i32(0).shape(), Ok(Shape::Null));
}

#[test]
fn point() {
    assert_eq!(
        Buf::default().i32(1).f64s(&[1.0, 2.0]).shape(),
        Ok(Shape::Point(Point { x: 1.0, y: 2.0 }))
    );
}

#[test]
fn point_m_no_data() {
    assert_eq!(
        Buf::default().i32(21).f64s(&[1.0, 2.0, -f64::MAX]).shape(),
        Ok(Shape::PointM(PointM {
            x: 1.0,
            y: 2.0,
            m: None
        }))
    );
}

#[test]
fn point_z() {
    assert_eq!(
        Buf::default().i32(11).f64s(&[1.0, 2.0, 3.0, 4.0]).shape(),
        Ok(Shape::PointZ(PointZ {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            m: Some(4.0)
        }))
    );
}

#[test]
fn multi_point() {
    assert_eq!(
        Buf::default()
            .i32(8)
            .f64s(&BBOX)
            .i32(2)
            .f64s(&[1.0, 2.0, 3.0, 4.0])
            .shape(),
        Ok(Shape::MultiPoint(MultiPoint {
            bbox: bbox(),
            points: points(&[[1.0, 2.0], [3.0, 4.0]]),
        }))
    );
}

#[test]
fn multi_point_m_without_optional_measures() {
    let shape = Buf::default()
        .i32(28)
        .f64s(&BBOX)
        .i32(1)
        .f64s(&[1.0, 2.0])
        .shape();
    assert_eq!(
        shape,
        Ok(Shape::MultiPointM(MultiPointM {
            xy: MultiPoint {
                bbox: bbox(),
                points: points(&[[1.0, 2.0]]),
            },
            m: None,
        }))
    );
}

#[test]
fn multi_point_z_with_measures() {
    let shape = Buf::default()
        .i32(18)
        .f64s(&BBOX)
        .i32(2)
        .f64s(&[1.0, 2.0, 3.0, 4.0])
        .f64s(&[5.0, 6.0, 5.0, 6.0])
        .f64s(&[7.0, 8.0, 7.0, -f64::MAX]);
    assert_eq!(
        shape.shape(),
        Ok(Shape::MultiPointZ(MultiPointZ {
            xy: MultiPoint {
                bbox: bbox(),
                points: points(&[[1.0, 2.0], [3.0, 4.0]]),
            },
            z: ZValues {
                range: Range { min: 5.0, max: 6.0 },
                values: vec![5.0, 6.0],
            },
            m: Some(Measures {
                range: Range {
                    min: Some(7.0),
                    max: Some(8.0)
                },
                values: vec![Some(7.0), None],
            }),
        }))
    );
}

/// Figure 2 of the specification: an outer ring with one hole, ten points.
#[test]
fn polygon_with_hole() {
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
    let shape = Buf::default()
        .i32(5)
        .f64s(&BBOX)
        .i32(2)
        .i32(10)
        .i32(0)
        .i32(5)
        .f64s(xy.as_flattened());
    assert_eq!(
        shape.shape(),
        Ok(Shape::Polygon(MultiPart {
            bbox: bbox(),
            parts: vec![0, 5],
            points: points(&xy),
        }))
    );
}

#[test]
fn poly_line_m_and_z_keep_their_type() {
    let line_m = Buf::default()
        .i32(23)
        .f64s(&BBOX)
        .i32(1)
        .i32(2)
        .i32(0)
        .f64s(&[0.0, 0.0, 1.0, 1.0]);
    assert_eq!(line_m.shape().map(|s| s.shape_type()), Ok(ShapeType::PolyLineM));

    let polygon_z = Buf::default()
        .i32(15)
        .f64s(&BBOX)
        .i32(1)
        .i32(4)
        .i32(0)
        .f64s(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0])
        .f64s(&[1.0, 4.0, 1.0, 2.0, 3.0, 4.0]);
    let Ok(Shape::PolygonZ(polygon)) = polygon_z.shape() else {
        panic!("expected PolygonZ");
    };
    assert_eq!(polygon.z.values, vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(polygon.m, None);
}

#[test]
fn multi_patch() -> Result<(), ShapeError> {
    let shape = Buf::default()
        .i32(31)
        .f64s(&BBOX)
        .i32(2)
        .i32(6)
        .i32(0)
        .i32(3)
        .i32(0)
        .i32(5)
        .f64s(&[0.0; 12])
        .f64s(&[0.0; 8])
        .shape()?;
    let Shape::MultiPatch(patch) = shape else {
        panic!("expected MultiPatch, got {shape:?}");
    };
    assert_eq!(patch.parts, vec![0, 3]);
    assert_eq!(patch.part_types, vec![PartType::TriangleStrip, PartType::Ring]);
    assert_eq!(patch.z.values.len(), 6);
    assert_eq!(patch.m, None);
    Ok(())
}

#[test]
fn shape_buffers_round_trip() -> Result<(), ShapeError> {
    let multi_point = MultiPoint {
        bbox: bbox(),
        points: points(&[[1.0, 2.0], [3.0, 4.0]]),
    };
    let multi_part = MultiPart {
        bbox: bbox(),
        parts: vec![0, 2],
        points: points(&[[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]),
    };
    let z = ZValues {
        range: Range { min: 1.0, max: 4.0 },
        values: vec![1.0, 2.0, 3.0, 4.0],
    };
    let m = Some(Measures {
        range: Range {
            min: Some(0.0),
            max: Some(2.0),
        },
        values: vec![Some(0.0), None, Some(2.0), None],
    });
    let shapes = [
        Shape::Null,
        Shape::Point(Point { x: 1.0, y: 2.0 }),
        Shape::PointM(PointM {
            x: 1.0,
            y: 2.0,
            m: None,
        }),
        Shape::PointZ(PointZ {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            m: Some(4.0),
        }),
        Shape::MultiPoint(multi_point.clone()),
        Shape::MultiPointM(MultiPointM {
            xy: multi_point.clone(),
            m: None,
        }),
        Shape::MultiPointZ(MultiPointZ {
            xy: multi_point,
            z: ZValues {
                range: Range { min: 1.0, max: 2.0 },
                values: vec![1.0, 2.0],
            },
            m: Some(Measures {
                range: Range {
                    min: Some(1.0),
                    max: Some(1.0),
                },
                values: vec![Some(1.0), None],
            }),
        }),
        Shape::PolyLine(multi_part.clone()),
        Shape::Polygon(multi_part.clone()),
        Shape::PolyLineM(MultiPartM {
            xy: multi_part.clone(),
            m: m.clone(),
        }),
        Shape::PolygonM(MultiPartM {
            xy: multi_part.clone(),
            m: None,
        }),
        Shape::PolyLineZ(MultiPartZ {
            xy: multi_part.clone(),
            z: z.clone(),
            m: None,
        }),
        Shape::PolygonZ(MultiPartZ {
            xy: multi_part.clone(),
            z: z.clone(),
            m: m.clone(),
        }),
        Shape::MultiPatch(MultiPatch {
            bbox: bbox(),
            parts: vec![0, 2],
            part_types: vec![PartType::TriangleFan, PartType::OuterRing],
            points: multi_part.points,
            z,
            m,
        }),
    ];
    for shape in &shapes {
        let bytes = Vec::<u8>::try_from(shape)?;
        assert_eq!(Shape::try_from(bytes.as_slice())?, *shape);
    }
    Ok(())
}

#[test]
fn errors() {
    let mismatched_z = Shape::MultiPointZ(MultiPointZ {
        xy: MultiPoint {
            bbox: bbox(),
            points: points(&[[1.0, 2.0]]),
        },
        z: ZValues {
            range: Range { min: 0.0, max: 0.0 },
            values: Vec::new(),
        },
        m: None,
    });
    assert_eq!(Vec::<u8>::try_from(&mismatched_z), Err(ShapeError::Corrupted));
    assert_eq!(
        Buf::default().i32(2).shape(),
        Err(ShapeError::InvalidShapeType(2))
    );
    assert_eq!(
        Buf::default().i32(1).f64s(&[1.0]).shape(),
        Err(ShapeError::UnexpectedEof)
    );
    let descending_parts = Buf::default()
        .i32(3)
        .f64s(&BBOX)
        .i32(2)
        .i32(4)
        .i32(2)
        .i32(0);
    assert_eq!(descending_parts.shape(), Err(ShapeError::Corrupted));
    let part_past_points = Buf::default().i32(3).f64s(&BBOX).i32(1).i32(1).i32(5);
    assert_eq!(part_past_points.shape(), Err(ShapeError::Corrupted));
    assert_eq!(
        ShapeFile::try_from([0u8; 100].as_slice()).map(|f| f.header),
        Err(ShapeError::InvalidFileCode(0))
    );
}
