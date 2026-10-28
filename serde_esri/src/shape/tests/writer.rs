use super::*;
use crate::shape::{
    error::FileError,
    file::ShapeFile,
    types::{PointZ, Range, ShapeType},
    writer::ShapeWriter,
};

#[test]
fn shape_writer() -> Result<(), FileError> {
    let shapes = [
        Shape::PointZ(PointZ {
            x: 1.0,
            y: 5.0,
            z: 2.0,
            m: None,
        }),
        Shape::Null,
        Shape::PointZ(PointZ {
            x: -3.0,
            y: 4.0,
            z: 7.0,
            m: Some(9.0),
        }),
    ];
    let mut writer = ShapeWriter::new(std::io::Cursor::new(Vec::new()), ShapeType::PointZ)?;
    for shape in &shapes {
        writer.write(shape)?;
    }
    let finished = writer.finish()?;
    let bytes = finished.writer.into_inner();

    let file = ShapeFile::try_from(bytes.as_slice())?;
    assert_eq!(file.header.file_length as usize * 2, bytes.len());
    assert_eq!(
        file.header.bbox,
        BoundingBox {
            xmin: -3.0,
            ymin: 4.0,
            xmax: 1.0,
            ymax: 5.0
        }
    );
    assert_eq!(file.header.z_range, Range { min: 2.0, max: 7.0 });
    assert_eq!(
        file.header.m_range,
        Range {
            min: Some(9.0),
            max: Some(9.0)
        }
    );
    let records = file.collect::<Result<Vec<_>, _>>()?;
    let numbers = records.iter().map(|r| r.number).collect::<Vec<_>>();
    assert_eq!(numbers, vec![1, 2, 3]);
    let written = records.into_iter().map(|r| r.shape).collect::<Vec<_>>();
    assert_eq!(written, shapes);

    let mut mixed = ShapeWriter::new(std::io::Cursor::new(Vec::new()), ShapeType::Point)?;
    assert!(matches!(
        mixed.write(&shapes[0]),
        Err(FileError::Shape(ShapeError::MixedShapeTypes {
            expected: ShapeType::Point,
            found: ShapeType::PointZ
        }))
    ));
    Ok(())
}

