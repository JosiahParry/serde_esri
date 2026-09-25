use super::*;

#[derive(Default)]
struct Buf(Vec<u8>);

impl Buf {
    fn i32(mut self, v: i32) -> Self {
        self.0.extend(v.to_le_bytes());
        self
    }

    fn u32(self, v: u32) -> Self {
        self.i32(v as i32)
    }

    fn f64s(mut self, vs: &[f64]) -> Self {
        for v in vs {
            self.0.extend(v.to_le_bytes());
        }
        self
    }

    fn read(&self) -> Result<Option<Geometry>, ShapeError> {
        EsriShape(&self.0).try_into()
    }
}

#[test]
fn null_shape() {
    assert_eq!(Buf::default().i32(0).read(), Ok(None));
}

#[test]
fn point() {
    let vertex = Vertex {
        x: 1.0,
        y: 2.0,
        ..Default::default()
    };
    assert_eq!(
        Buf::default().i32(1).f64s(&[1.0, 2.0]).read(),
        Ok(Some(Geometry::Point(Point(Some(vertex)))))
    );
}

#[test]
fn empty_point() {
    assert_eq!(
        Buf::default().i32(1).f64s(&[f64::NAN, f64::NAN]).read(),
        Ok(Some(Geometry::Point(Point(None))))
    );
}

#[test]
fn general_point_with_zm_ids() -> Result<(), ShapeError> {
    let geometry = Buf::default()
        .u32(52 | HAS_ZS | HAS_MS | HAS_IDS)
        .f64s(&[1.0, 2.0, 3.0, -f64::MAX])
        .i32(7)
        .read()?;
    let Some(Geometry::Point(Point(Some(v)))) = geometry else {
        panic!("expected point, got {geometry:?}");
    };
    assert_eq!((v.x, v.y, v.z, v.id), (1.0, 2.0, Some(3.0), Some(7)));
    assert!(v.m.is_some_and(f64::is_nan));
    Ok(())
}

#[test]
fn multipoint_z() -> Result<(), ShapeError> {
    let geometry = Buf::default()
        .i32(20)
        .f64s(&[0.0, 0.0, 1.0, 1.0])
        .i32(2)
        .f64s(&[0.0, 0.0, 1.0, 1.0])
        .f64s(&[5.0, 6.0, 5.0, 6.0])
        .read()?;
    let Some(Geometry::MultiPoint(mp)) = geometry else {
        panic!("expected multipoint, got {geometry:?}");
    };
    assert_eq!(mp.vertices.xy, vec![[0.0, 0.0], [1.0, 1.0]]);
    assert_eq!(mp.vertices.z, Some(vec![5.0, 6.0]));
    assert_eq!(mp.vertices.m, None);
    Ok(())
}

#[test]
fn polygon_drops_closing_vertex_and_keeps_open_ring() -> Result<(), ShapeError> {
    // Ring 0 is closed (4 points), ring 1 is not closed (3 points).
    let geometry = Buf::default()
        .i32(25)
        .f64s(&[0.0, 0.0, 10.0, 10.0])
        .i32(2)
        .i32(7)
        .i32(0)
        .i32(4)
        .f64s(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0])
        .f64s(&[5.0, 5.0, 6.0, 5.0, 6.0, 6.0])
        .f64s(&[0.0, 6.0])
        .f64s(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
        .read()?;
    let Some(Geometry::Polygon(Polygon { rings, .. })) = geometry else {
        panic!("expected polygon, got {geometry:?}");
    };
    assert_eq!(rings.path_offsets, vec![0, 3, 6]);
    assert_eq!(
        rings.vertices.xy,
        vec![
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [5.0, 5.0],
            [6.0, 5.0],
            [6.0, 6.0]
        ]
    );
    assert_eq!(rings.vertices.m, Some(vec![0.0, 1.0, 2.0, 4.0, 5.0, 6.0]));
    assert!(rings.is_closed_path(0) && rings.is_closed_path(1));
    Ok(())
}

#[test]
fn polyline_collapses_empty_parts() -> Result<(), ShapeError> {
    let geometry = Buf::default()
        .i32(3)
        .f64s(&[0.0, 0.0, 3.0, 3.0])
        .i32(3)
        .i32(4)
        .i32(0)
        .i32(2)
        .i32(2)
        .f64s(&[0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0])
        .read()?;
    let Some(Geometry::Polyline(Polyline(path))) = geometry else {
        panic!("expected polyline, got {geometry:?}");
    };
    assert_eq!(path.path_offsets, vec![0, 2, 4]);
    assert_eq!(path.path_flags, vec![PathFlags::default(); 2]);
    Ok(())
}

#[test]
fn errors() {
    assert_eq!(
        Buf::default().u32(51 | HAS_CURVES).read(),
        Err(ShapeError::UnsupportedCurves)
    );
    assert_eq!(
        Buf::default().i32(32).read(),
        Err(ShapeError::InvalidShapeType(32))
    );
    assert_eq!(
        Buf::default().i32(1).f64s(&[1.0]).read(),
        Err(ShapeError::UnexpectedEof)
    );
    assert_eq!(
        Buf::default()
            .i32(3)
            .f64s(&[0.0; 4])
            .i32(2)
            .i32(2)
            .i32(1)
            .i32(0)
            .read(),
        Err(ShapeError::Corrupted)
    );
}
