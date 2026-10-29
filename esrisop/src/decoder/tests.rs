use super::*;
use crate::xz::MaxLevel;
use arrow_array::{
    ArrayRef, BinaryArray, Float64Array, Int32Array, Int64Array, StringArray, StructArray,
};
use arrow_schema::{DataType, Field};
use esripbf::esri_p_buffer::feature_collection_p_buffer as pbf;
use parquet::{
    arrow::{arrow_reader::ArrowReaderOptions, ArrowWriter},
    file::{
        metadata::{KeyValue, PageIndexPolicy, ParquetMetaDataReader},
        properties::WriterProperties,
    },
    schema::types::ColumnPath,
};
use prost::Message;
use serde_esri::enginex::{
    geometry::{Geometry, Point},
    shape::EsriShape,
};

pub(crate) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SPARK_KEY: &str = "org.apache.spark.sql.parquet.row.metadata";

fn write(batch: &RecordBatch, key: &str, value: &str, rows_per_group: usize) -> TestResult<Bytes> {
    let properties = WriterProperties::builder()
        .set_max_row_group_row_count(Some(rows_per_group))
        .set_data_page_row_count_limit(1)
        .set_write_batch_size(1)
        .set_key_value_metadata(Some(vec![KeyValue::new(
            key.to_string(),
            value.to_string(),
        )]))
        .build();
    let mut buffer = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buffer, batch.schema(), Some(properties))?;
    writer.write(batch)?;
    writer.close()?;
    Ok(Bytes::from(buffer))
}

pub(crate) fn metadata(bytes: &Bytes) -> TestResult<ArrowReaderMetadata> {
    let metadata = ParquetMetaDataReader::new()
        .with_page_index_policy(PageIndexPolicy::Optional)
        .parse_and_finish(bytes)?;
    Ok(ArrowReaderMetadata::try_new(
        Arc::new(metadata),
        ArrowReaderOptions::new(),
    )?)
}

pub(crate) fn read(bytes: &Bytes, query: &Query) -> TestResult<(SchemaRef, Vec<RecordBatch>)> {
    let mut decoder = Decoder::try_new(metadata(bytes)?, query)?;
    let mut batches = Vec::new();
    loop {
        match decoder.try_decode()? {
            DecodeResult::NeedsData(ranges) => {
                let data = ranges
                    .iter()
                    .map(|r| bytes.slice(r.start as usize..r.end as usize))
                    .collect();
                decoder.push_ranges(ranges, data)?;
            }
            DecodeResult::Data(batch) => batches.push(batch),
            DecodeResult::Finished => break,
        }
    }
    Ok((decoder.schema().clone(), batches))
}

fn field_names(schema: &SchemaRef) -> Vec<&str> {
    schema.fields().iter().map(|f| f.name().as_str()).collect()
}

fn names(batches: &[RecordBatch]) -> Vec<String> {
    let mut names = Vec::new();
    for batch in batches {
        if let Some(column) = batch
            .column_by_name("name")
            .and_then(|c| c.as_string_opt::<i32>())
        {
            names.extend(column.iter().flatten().map(str::to_string));
        }
    }
    names
}

fn geometries(batches: &[RecordBatch]) -> TestResult<Vec<Option<Geometry>>> {
    let mut geometries = Vec::new();
    for batch in batches {
        let column = batch
            .column_by_name("geometry")
            .and_then(|c| c.as_binary_opt::<i32>())
            .ok_or("no binary geometry column")?;
        for bytes in column {
            geometries.push(match bytes {
                Some(bytes) => Option::<Geometry>::try_from(EsriShape(bytes))?,
                None => None,
            });
        }
    }
    Ok(geometries)
}

fn extent(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Envelope2D {
    Envelope2D {
        xmin,
        ymin,
        xmax,
        ymax,
    }
}

fn square(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<u8> {
    pbf::Geometry {
        lengths: vec![5],
        coords: vec![x0, y0, 0, y1 - y0, x1 - x0, 0, 0, y0 - y1, x0 - x1, 0],
        ..Default::default()
    }
    .encode_to_vec()
}

const XZ: &str = r#"{"version":"0.1","type":"xz","wkid":3857,"geometryType":"polygon","maxLevel":2,"hasZ":false,"hasM":false,"code":["geodisplay","xz-code"],"encoding":"esriPBF","fullExtent":{"xmin":0,"ymin":0,"xmax":10,"ymax":10},"levels":[{"column":["geodisplay","multiscale-0"],"level":0,"resolution":0.01,"scale":1,"transform":{"scale":[0.01,0.01,1,1],"translate":[0,0,0,0]}}]}"#;

fn polygons() -> TestResult<Bytes> {
    let shapes = [
        square(0, 0, 1000, 1000),
        square(800, 800, 900, 900),
        square(50, 50, 150, 150),
        square(800, 800, 900, 900),
    ];
    let geodisplay = StructArray::from(vec![
        (
            Arc::new(Field::new("xz-code", DataType::Int64, false)),
            Arc::new(Int64Array::from(vec![0, 0, 2, 20])) as ArrayRef,
        ),
        (
            Arc::new(Field::new("multiscale-0", DataType::Binary, true)),
            Arc::new(BinaryArray::from_iter_values(shapes.iter())) as ArrayRef,
        ),
    ]);
    let batch = RecordBatch::try_from_iter([
        (
            "name",
            Arc::new(StringArray::from(vec!["c", "d", "a", "b"])) as ArrayRef,
        ),
        ("geodisplay", Arc::new(geodisplay) as ArrayRef),
    ])?;
    write(&batch, "geodisplay", XZ, 1)
}

#[test]
fn xz_query_prunes_row_groups_and_tests_extents() -> TestResult<()> {
    let bytes = polygons()?;
    let query = Query {
        extents: vec![extent(0.0, 0.0, 1.0, 1.0)],
        resolution: Some(0.01),
        columns: None,
    };

    let metadata = metadata(&bytes)?;
    let ranges = XzRanges::new(
        &extent(0.0, 0.0, 10.0, 10.0),
        &query.extents,
        MaxLevel::try_from(2)?,
    );
    let index_leaf = ColumnPath::new(vec!["geodisplay".into(), "xz-code".into()])
        .leaf(metadata.parquet_schema())?;
    assert_eq!(ranges.0, vec![0..=2]);
    let parquet_metadata = metadata.metadata();
    let plan = Plan::new(parquet_metadata, |row_group, index| {
        i64::rows(
            parquet_metadata,
            row_group,
            index,
            index_leaf,
            |[min, max]| ranges.overlaps(min, max),
        )
    });
    assert_eq!(plan.row_groups, vec![0, 1, 2]);

    let (schema, batches) = read(&bytes, &query)?;
    assert_eq!(field_names(&schema), vec!["name", "geometry"]);
    assert_eq!(names(&batches), vec!["c", "a"]);
    Ok(())
}

#[test]
fn xz_without_resolution_filters_by_code_only() -> TestResult<()> {
    let query = Query {
        extents: vec![extent(0.0, 0.0, 1.0, 1.0)],
        ..Default::default()
    };
    let (schema, batches) = read(&polygons()?, &query)?;
    assert_eq!(field_names(&schema), vec!["name"]);
    assert_eq!(names(&batches), vec!["c", "d", "a"]);
    Ok(())
}

#[test]
fn reads_every_row_without_extents() -> TestResult<()> {
    let query = Query {
        resolution: Some(0.01),
        ..Default::default()
    };
    let (schema, batches) = read(&polygons()?, &query)?;
    assert_eq!(names(&batches), vec!["c", "d", "a", "b"]);

    let field = schema.field_with_name("geometry")?;
    assert_eq!(field.data_type(), &DataType::Binary);
    let metadata = field.metadata();
    assert_eq!(
        metadata.get("esri.encoding").map(String::as_str),
        Some("EsriShape")
    );
    assert_eq!(
        metadata.get("esri.sr_wkt").map(String::as_str),
        Some(r#"{"wkid":3857}"#)
    );
    assert_eq!(metadata.get("geodisplay").map(String::as_str), Some(XZ));

    let geometries = geometries(&batches)?;
    assert_eq!(geometries.len(), 4);
    let Some(Some(Geometry::Polygon(first))) = geometries.first() else {
        return Err("expected a polygon".into());
    };
    assert_eq!(
        first.rings.vertices.xy,
        vec![[0.0, 0.0], [0.0, 10.0], [10.0, 10.0], [10.0, 0.0]]
    );
    Ok(())
}

#[test]
fn reads_requested_columns_only() -> TestResult<()> {
    let bytes = polygons()?;
    let query = Query {
        resolution: Some(0.01),
        columns: Some(vec!["geodisplay".into()]),
        ..Default::default()
    };
    let (schema, _) = read(&bytes, &query)?;
    assert_eq!(field_names(&schema), vec!["geodisplay", "geometry"]);

    let missing = Query {
        columns: Some(vec!["nope".into()]),
        ..Default::default()
    };
    assert!(read(&bytes, &missing).is_err_and(|e| e.to_string().contains("`nope`")));
    Ok(())
}

const SPARK_Z: &str = r#"{"type":"struct","fields":[{"name":"OBJECTID","type":"integer","nullable":false,"metadata":{}},{"name":"geodisplay","type":{"type":"struct","fields":[{"name":"indexkey","type":"long","nullable":false,"metadata":{}},{"name":"xcoord","type":"double","nullable":false,"metadata":{}},{"name":"ycoord","type":"double","nullable":false,"metadata":{}}]},"nullable":true,"metadata":{"coordinatePrecision":16,"version":"0.1","fullExtent":{"xmin":-180,"ymin":-90,"xmax":180,"ymax":90},"type":"z","xColumn":"xcoord","yColumn":"ycoord","wkid":4326,"code":"indexkey"}}]}"#;

fn spark_points() -> TestResult<Bytes> {
    let geodisplay = StructArray::from(vec![
        (
            Arc::new(Field::new("indexkey", DataType::Int64, false)),
            Arc::new(Int64Array::from(vec![0, 1, 2, 3, 4])) as ArrayRef,
        ),
        (
            Arc::new(Field::new("xcoord", DataType::Float64, false)),
            Arc::new(Float64Array::from(vec![
                -170.0,
                0.0,
                170.0,
                179.0,
                f64::NAN,
            ])) as ArrayRef,
        ),
        (
            Arc::new(Field::new("ycoord", DataType::Float64, false)),
            Arc::new(Float64Array::from(vec![0.0, 0.0, 10.0, -5.0, f64::NAN])) as ArrayRef,
        ),
    ]);
    let batch = RecordBatch::try_from_iter([
        (
            "OBJECTID",
            Arc::new(Int32Array::from(vec![1, 2, 3, 4, 5])) as ArrayRef,
        ),
        ("geodisplay", Arc::new(geodisplay) as ArrayRef),
    ])?;
    write(&batch, SPARK_KEY, SPARK_Z, 2)
}

fn ids(batches: &[RecordBatch]) -> Vec<i32> {
    batches
        .iter()
        .filter_map(|b| b.column_by_name("OBJECTID"))
        .filter_map(|c| c.as_primitive_opt::<arrow_array::types::Int32Type>())
        .flat_map(|c| c.values().to_vec())
        .collect()
}

#[test]
fn z_query_across_the_antimeridian() -> TestResult<()> {
    let query = Query {
        extents: vec![
            extent(170.0, -90.0, 180.0, 90.0),
            extent(-180.0, -90.0, -165.0, 90.0),
        ],
        ..Default::default()
    };
    let (schema, batches) = read(&spark_points()?, &query)?;
    assert_eq!(field_names(&schema), vec!["OBJECTID", "geometry"]);
    assert_eq!(ids(&batches), vec![1, 3, 4]);
    let points = geometries(&batches)?
        .into_iter()
        .map(|g| match g {
            Some(Geometry::Point(Point(Some(v)))) => Some([v.x, v.y]),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        points,
        vec![
            Some([-170.0, 0.0]),
            Some([170.0, 10.0]),
            Some([179.0, -5.0])
        ]
    );
    Ok(())
}

#[test]
fn z_points_with_nan_coordinates_are_null() -> TestResult<()> {
    let (schema, batches) = read(&spark_points()?, &Query::default())?;
    let field = schema.field_with_name("geometry")?;
    let raw = field
        .metadata()
        .get("geodisplay")
        .ok_or("no geodisplay metadata")?;
    assert!(raw.starts_with(r#"{"coordinatePrecision":16"#));
    assert_eq!(ids(&batches), vec![1, 2, 3, 4, 5]);
    let geometries = geometries(&batches)?;
    assert_eq!(geometries.iter().filter(|g| g.is_none()).count(), 1);
    Ok(())
}
