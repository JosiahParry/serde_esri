use super::*;
use crate::shape::{
    error::FileError,
    file::{FileHeader, ShapeFile},
    reader::ShapeReader,
    types::{Range, ShapeType},
};

#[test]
fn shape_file() -> Result<(), FileError> {
    let mut file = Buf::default()
        .i32_be(9994)
        .f64s(&[0.0; 2])
        .i32(0)
        .i32_be(0)
        .i32(1000)
        .i32(1)
        .f64s(&BBOX)
        .f64s(&[0.0, 0.0, -f64::MAX, -f64::MAX]);
    for (number, x) in [(1, 1.0), (2, 2.0)] {
        file = file.i32_be(number).i32_be(10).i32(1).f64s(&[x, 0.0]);
    }
    file = file.i32_be(3).i32_be(2).i32(0);

    let shapes = ShapeFile::try_from(file.0.as_slice())?;
    assert_eq!(shapes.header.version, 1000);
    assert_eq!(shapes.header.shape_type, ShapeType::Point);
    assert_eq!(shapes.header.m_range, Range { min: None, max: None });

    let records = shapes.collect::<Result<Vec<_>, _>>()?;
    let numbers = records.iter().map(|r| r.number).collect::<Vec<_>>();
    assert_eq!(numbers, vec![1, 2, 3]);
    assert_eq!(records[1].shape, Shape::Point(Point { x: 2.0, y: 0.0 }));
    assert_eq!(records[2].shape, Shape::Null);

    let mut streamed = ShapeReader::new(file.0.as_slice())?;
    assert_eq!(streamed.header, FileHeader::try_from(file.0.as_slice())?);
    assert_eq!(streamed.by_ref().collect::<Result<Vec<_>, _>>()?, records);
    assert!(streamed.next().is_none());

    let truncated = ShapeReader::new(&file.0[..file.0.len() - 2])?;
    let results = truncated.collect::<Vec<_>>();
    assert_eq!(results.len(), 3);
    assert!(matches!(
        results[2],
        Err(FileError::Shape(ShapeError::UnexpectedEof))
    ));
    Ok(())
}

