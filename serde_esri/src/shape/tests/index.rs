use super::*;
use crate::shape::{
    error::FileError,
    index::ShapeIndex,
    reader::ShapeReader,
    types::ShapeType,
    writer::{FinishedShapes, ShapeWriter},
};

#[test]
fn shape_index_and_random_access() -> Result<(), FileError> {
    let shapes = (0..4)
        .map(|i| match i {
            2 => Shape::Null,
            _ => Shape::Point(Point {
                x: f64::from(i),
                y: 0.0,
            }),
        })
        .collect::<Vec<_>>();
    let mut writer = ShapeWriter::new(std::io::Cursor::new(Vec::new()), ShapeType::Point)?;
    for shape in &shapes {
        writer.write(shape)?;
    }
    let FinishedShapes { writer, index } = writer.finish()?;
    let shp = writer.into_inner();

    // Point records are 8 + 20 bytes (14 words); the null record is 8 + 4 bytes (6 words).
    let offsets = index.records.iter().map(|r| r.offset).collect::<Vec<_>>();
    assert_eq!(offsets, vec![50, 64, 78, 84]);
    assert_eq!(index.header.file_length, 50 + 4 * 4);
    let shx = Vec::<u8>::from(&index);
    assert_eq!(shx.len(), index.header.file_length as usize * 2);
    assert_eq!(ShapeIndex::try_from(shx.as_slice())?, index);

    let readers = [
        ShapeReader::new(std::io::Cursor::new(shp.as_slice()))?,
        ShapeReader::new(std::io::Cursor::new(shp.as_slice()))?.with_index(index),
    ];
    for mut reader in readers {
        let third = reader.read_nth(3).transpose()?.map(|r| r.shape);
        assert_eq!(third.as_ref(), shapes.get(3));
        assert!(reader.next().is_none());

        let first = reader.read_nth(0).transpose()?.map(|r| r.number);
        assert_eq!(first, Some(1));
        let rest = reader.by_ref().collect::<Result<Vec<_>, _>>()?;
        assert_eq!(rest.len(), 3);

        reader.seek(4)?;
        assert!(reader.next().is_none());
        assert!(reader.read_nth(4).is_none());
        assert!(matches!(
            reader.seek(5),
            Err(FileError::Shape(ShapeError::NoSuchRecord(5)))
        ));
    }
    Ok(())
}

