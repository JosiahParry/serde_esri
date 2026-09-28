//! Tests for [`crate::shape`], sharing buffer-building fixtures.

use super::*;

mod buffer;
mod file;
mod index;
mod writer;

#[derive(Default)]
struct Buf(Vec<u8>);

impl Buf {
    fn i32(mut self, v: i32) -> Self {
        self.0.extend(v.to_le_bytes());
        self
    }

    fn i32_be(mut self, v: i32) -> Self {
        self.0.extend(v.to_be_bytes());
        self
    }

    fn f64s(mut self, vs: &[f64]) -> Self {
        for v in vs {
            self.0.extend(v.to_le_bytes());
        }
        self
    }

    fn shape(&self) -> Result<Shape, ShapeError> {
        Shape::try_from(self.0.as_slice())
    }
}

const BBOX: [f64; 4] = [0.0, 0.0, 10.0, 10.0];

fn bbox() -> BoundingBox {
    BoundingBox {
        xmin: 0.0,
        ymin: 0.0,
        xmax: 10.0,
        ymax: 10.0,
    }
}

fn points(xy: &[[f64; 2]]) -> Vec<Point> {
    xy.iter().map(|&[x, y]| Point { x, y }).collect()
}
