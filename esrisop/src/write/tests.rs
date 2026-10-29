use super::*;
use crate::{
    decoder::tests::{read, TestResult},
    query::Query,
};
use arrow_array::{
    cast::AsArray,
    types::{Int32Type, Int64Type},
    BinaryArray, Int32Array,
};
use bytes::Bytes;
use parquet::arrow::ArrowWriter;
use serde_esri::enginex::{
    geometry::{MultiPath, MultiPoint, Polygon},
    shape::writer::EsriShapeBuffer,
    vertex::VertexAttributes,
};

fn square(x: f64, y: f64, size: f64) -> Option<Geometry> {
    Some(Geometry::Polygon(Polygon::from(MultiPath {
        vertices: VertexAttributes {
            xy: vec![[x, y], [x, y + size], [x + size, y + size], [x + size, y]],
            ..Default::default()
        },
        path_offsets: vec![0, 4],
        path_flags: vec![Default::default()],
        segments: None,
    })))
}

fn point(x: f64, y: f64, z: f64) -> Option<Geometry> {
    Some(Geometry::Point(Point(Some(Vertex {
        x,
        y,
        z: Some(z),
        ..Default::default()
    }))))
}

fn table(geometries: &[Option<Geometry>]) -> TestResult<RecordBatch> {
    let shapes = geometries
        .iter()
        .map(|geometry| {
            geometry
                .as_ref()
                .map(|geometry| EsriShapeBuffer::try_from(geometry).map(|shape| shape.0))
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ids = Int32Array::from_iter_values(0..geometries.len() as i32);
    Ok(RecordBatch::try_from_iter([
        ("id", Arc::new(ids) as ArrayRef),
        (
            "geometry",
            Arc::new(BinaryArray::from_iter(shapes)) as ArrayRef,
        ),
    ])?)
}

fn optimization(wkid: i32) -> DisplayOptimization {
    DisplayOptimization {
        geometry_column: "geometry".to_string(),
        wkid,
        full_extent: None,
    }
}

fn file(optimized: &Optimized, rows_per_group: usize) -> TestResult<Bytes> {
    let properties = optimized
        .properties()?
        .set_max_row_group_row_count(Some(rows_per_group))
        .build();
    let mut buffer = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buffer, optimized.batch.schema(), Some(properties))?;
    writer.write(&optimized.batch)?;
    writer.close()?;
    Ok(Bytes::from(buffer))
}

fn ids(batches: &[RecordBatch]) -> Vec<i32> {
    let mut ids = batches
        .iter()
        .filter_map(|batch| batch.column_by_name("id")?.as_primitive_opt::<Int32Type>())
        .flat_map(|ids| ids.values().iter().copied())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn shapes(batches: &[RecordBatch]) -> TestResult<Vec<Geometry>> {
    let mut shapes = Vec::new();
    for batch in batches {
        let column = batch
            .column_by_name("geometry")
            .and_then(|column| column.as_binary_opt::<i32>())
            .ok_or("no geometry column")?;
        for bytes in column.iter().flatten() {
            shapes.extend(Option::<Geometry>::try_from(EsriShape(bytes))?);
        }
    }
    Ok(shapes)
}

fn group(optimized: &Optimized) -> TestResult<&StructArray> {
    Ok(optimized
        .batch
        .column_by_name(GROUP)
        .and_then(|column| column.as_struct_opt())
        .ok_or("no geodisplay column")?)
}

#[test]
fn polygons_sort_by_index_key_and_read_back_through_a_bbox() -> TestResult {
    let geometries = [
        square(10.0, 10.0, 1.0),
        square(-100.0, 40.0, 0.5),
        None,
        square(-99.5, 40.2, 0.1),
        square(120.0, -30.0, 2.0),
        square(10.5, 10.5, 0.01),
    ];
    let optimized = optimization(4326).apply(&table(&geometries)?)?;
    let GeodisplayMetadata::Xz(xz) = &optimized.geodisplay else {
        return Err("expected an XZ index".into());
    };
    assert_eq!(xz.display.geometry_type, Some(GeometryType::Polygon));
    assert_eq!(xz.levels.len(), 9);
    let full = &xz.display.full_extent;
    assert_eq!(
        [full.xmin, full.ymin, full.xmax, full.ymax],
        [-100.0, -30.0, 122.0, 40.5]
    );
    let keys = group(&optimized)?
        .column_by_name("indexkey")
        .and_then(|keys| keys.as_primitive_opt::<Int64Type>())
        .ok_or("no index keys")?;
    assert!(keys.values().windows(2).all(|pair| pair[0] <= pair[1]));

    let bytes = file(&optimized, 2)?;
    let query = Query {
        extents: vec![Envelope2D {
            xmin: -100.2,
            ymin: 40.1,
            xmax: -99.45,
            ymax: 40.25,
        }],
        resolution: Some(0.001),
        columns: Some(vec!["id".to_string()]),
    };
    let (_, batches) = read(&bytes, &query)?;
    assert_eq!(ids(&batches), vec![1, 3]);
    let envelopes = shapes(&batches)?
        .iter()
        .filter_map(|shape| shape.envelope().xy)
        .map(|xy| [xy.xmin, xy.ymin, xy.xmax, xy.ymax])
        .collect::<Vec<_>>();
    assert_eq!(envelopes.len(), 2);
    for envelope in envelopes {
        let expected = if envelope[0] < -99.75 {
            [-100.0, 40.0, -99.5, 40.5]
        } else {
            [-99.5, 40.2, -99.4, 40.3]
        };
        for (found, expected) in envelope.iter().zip(expected) {
            assert!((found - expected).abs() <= 0.001, "{envelope:?}");
        }
    }
    Ok(())
}

#[test]
fn a_geometry_that_collapses_at_a_level_keeps_one_vertex() -> TestResult {
    let geometries = [square(10.0, 10.0, 0.00001), square(-50.0, -50.0, 30.0)];
    let optimized = optimization(4326).apply(&table(&geometries)?)?;
    let level = group(&optimized)?
        .column_by_name("esri-multiscale-0")
        .and_then(|level| level.as_binary_opt::<i32>())
        .ok_or("no level 0 column")?;
    let lengths = level
        .iter()
        .flatten()
        .map(|bytes| pbf::Geometry::decode(bytes).map(|geometry| geometry.lengths))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(lengths.contains(&vec![1]));
    assert!(lengths.contains(&vec![5]));
    Ok(())
}

#[test]
fn points_keep_z_and_read_back_through_a_bbox() -> TestResult {
    let geometries = [
        point(1.0, 1.0, 5.0),
        point(50.0, 20.0, 6.0),
        point(-60.0, -10.0, 7.0),
        point(2.0, 2.0, 8.0),
    ];
    let optimized = optimization(4326).apply(&table(&geometries)?)?;
    let GeodisplayMetadata::Z(z) = &optimized.geodisplay else {
        return Err("expected a Z index".into());
    };
    assert!(z.display.has_z);
    assert_eq!(
        z.z_column,
        Some(ColumnPath::new(vec![
            GROUP.to_string(),
            "zcoord".to_string()
        ]))
    );

    let bytes = file(&optimized, 2)?;
    let query = Query {
        extents: vec![Envelope2D {
            xmin: 0.0,
            ymin: 0.0,
            xmax: 3.0,
            ymax: 3.0,
        }],
        resolution: None,
        columns: Some(vec!["id".to_string()]),
    };
    let (_, batches) = read(&bytes, &query)?;
    assert_eq!(ids(&batches), vec![0, 3]);
    let mut zs = shapes(&batches)?
        .iter()
        .filter_map(|shape| match shape {
            Geometry::Point(Point(vertex)) => vertex.and_then(|vertex| vertex.z),
            _ => None,
        })
        .collect::<Vec<_>>();
    zs.sort_by(f64::total_cmp);
    assert_eq!(zs, vec![5.0, 8.0]);
    Ok(())
}

#[test]
fn z_polylines_keep_only_the_original_vertices_douglas_peucker_keeps() -> TestResult {
    let vertices = (0..50)
        .map(|i| Vertex {
            x: f64::from(i) * 0.1,
            y: f64::from(i % 2) * 0.01,
            z: Some(f64::from(i)),
            ..Default::default()
        })
        .collect::<VertexAttributes>();
    let line = Some(Geometry::Polyline(Polyline(MultiPath {
        vertices,
        path_offsets: vec![0, 50],
        path_flags: vec![Default::default()],
        segments: None,
    })));
    let optimized = optimization(4326).apply(&table(&[line])?)?;
    let level = |name: &str| -> TestResult<pbf::Geometry> {
        let bytes = group(&optimized)?
            .column_by_name(name)
            .and_then(|level| level.as_binary_opt::<i32>())
            .and_then(|level| level.iter().flatten().next())
            .ok_or("no level geometry")?;
        Ok(pbf::Geometry::decode(bytes)?)
    };
    let coarse = level("esri-multiscale-0")?;
    assert_eq!(coarse.lengths, vec![2]);
    assert_eq!(coarse.coords, vec![0, 0, 0, 7, 0, 49]);
    assert_eq!(level("esri-multiscale-16")?.lengths, vec![50]);
    Ok(())
}

#[test]
fn rejects_mixed_types_other_spatial_references_and_an_existing_group() -> TestResult {
    let mixed = table(&[square(0.0, 0.0, 1.0), point(1.0, 1.0, 0.0)])?;
    assert!(matches!(
        optimization(4326).apply(&mixed),
        Err(Error::MixedGeometryTypes(
            GeometryType::Polygon,
            GeometryType::Point
        ))
    ));

    let squares = table(&[square(0.0, 0.0, 1.0)])?;
    assert!(matches!(
        optimization(2263).apply(&squares),
        Err(Error::Wkid(2263))
    ));

    let renamed = RecordBatch::try_from_iter([
        ("geodisplay", squares.column(0).clone()),
        ("geometry", squares.column(1).clone()),
    ])?;
    assert!(matches!(
        optimization(4326).apply(&renamed),
        Err(Error::ColumnExists(_))
    ));
    Ok(())
}

#[test]
fn multipoints_read_back_through_a_bbox_with_the_wkid_as_json() -> TestResult {
    let multipoint = |xy: &[[f64; 2]]| {
        Some(Geometry::MultiPoint(MultiPoint {
            vertices: VertexAttributes {
                xy: xy.to_vec(),
                ..Default::default()
            },
        }))
    };
    let geometries = [
        multipoint(&[[1.0, 1.0], [1.5, 1.5]]),
        multipoint(&[[50.0, 20.0], [51.0, 21.0], [52.0, 22.0]]),
        multipoint(&[[-60.0, -10.0]]),
    ];
    let optimized = optimization(4326).apply(&table(&geometries)?)?;
    assert_eq!(
        optimized.geodisplay.display().geometry_type,
        Some(GeometryType::MultiPoint)
    );

    let bytes = file(&optimized, 1)?;
    let query = Query {
        extents: vec![Envelope2D {
            xmin: 0.0,
            ymin: 0.0,
            xmax: 2.0,
            ymax: 2.0,
        }],
        resolution: Some(0.0001),
        columns: Some(vec!["id".to_string()]),
    };
    let (schema, batches) = read(&bytes, &query)?;
    assert_eq!(ids(&batches), vec![0]);
    let shapes = shapes(&batches)?;
    let [Geometry::MultiPoint(points)] = shapes.as_slice() else {
        return Err("expected one multipoint".into());
    };
    assert_eq!(points.vertices.len(), 2);
    let geometry = schema.field_with_name("geometry")?;
    assert_eq!(
        geometry.metadata().get("esri.sr_wkt").map(String::as_str),
        Some(r#"{"wkid":4326}"#)
    );
    Ok(())
}
