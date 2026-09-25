//! Native conversion of engine geometries into GeoArrow arrays with interleaved coordinates.
//!
//! Engine buffers map onto GeoArrow's directly: vertices become coordinates, path offsets become
//! line or ring offsets, and OGC polygon groups (see [`Polygon::ogc_polygons`]) become polygon
//! offsets. Rings and closed paths gain their closing vertex, and vertex IDs are dropped.
//!
//! ```ignore
//! let polygons: Vec<Option<Polygon>> = ...;
//! let array = MultiPolygonArray::try_from(GeometryColumn(&polygons))?;
//! ```

use crate::enginex::{
    Geometry, MultiPoint, Point, Polygon, Polyline, Semantics, Vertex, VertexAttributes,
    VertexDescription,
};
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use geoarrow_array::{
    array::{
        CoordBuffer, GeometryArray, InterleavedCoordBuffer, MultiLineStringArray, MultiPointArray,
        MultiPolygonArray, PointArray,
    },
    builder::GeometryBuilder,
};
use geoarrow_schema::{error::GeoArrowError, Dimension, GeometryType};
use std::ops::Range;

/// Engine geometries of one kind, with `None` for nulls, to convert into a GeoArrow array.
#[derive(Clone, Copy, Debug)]
pub struct GeometryColumn<'a, G>(pub &'a [Option<G>]);

#[derive(Debug)]
pub enum ToGeoArrowError {
    /// Geometries in the column carry different Z and M attributes.
    MixedDimensions,
    /// An offset does not fit GeoArrow's 32-bit offsets.
    TooLarge,
    GeoArrow(GeoArrowError),
}

impl std::fmt::Display for ToGeoArrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToGeoArrowError::MixedDimensions => {
                write!(f, "geometries in a column must share their dimensions")
            }
            ToGeoArrowError::TooLarge => write!(f, "offsets exceed GeoArrow's 32-bit limit"),
            ToGeoArrowError::GeoArrow(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ToGeoArrowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ToGeoArrowError::GeoArrow(e) => Some(e),
            _ => None,
        }
    }
}

impl From<GeoArrowError> for ToGeoArrowError {
    fn from(e: GeoArrowError) -> Self {
        ToGeoArrowError::GeoArrow(e)
    }
}

/// Z and M map to their dimensions; IDs have none.
impl From<VertexDescription> for Dimension {
    fn from(description: VertexDescription) -> Self {
        match (description.has(Semantics::Z), description.has(Semantics::M)) {
            (false, false) => Dimension::XY,
            (true, false) => Dimension::XYZ,
            (false, true) => Dimension::XYM,
            (true, true) => Dimension::XYZM,
        }
    }
}

impl<G> GeometryColumn<'_, G> {
    /// The dimension every described geometry shares, or XY when none are described.
    fn dimension(
        &self,
        description: fn(&G) -> Option<VertexDescription>,
    ) -> Result<Dimension, ToGeoArrowError> {
        let mut dims = self
            .0
            .iter()
            .flatten()
            .filter_map(description)
            .map(Dimension::from);
        let Some(first) = dims.next() else {
            return Ok(Dimension::XY);
        };
        if dims.any(|dim| dim != first) {
            return Err(ToGeoArrowError::MixedDimensions);
        }
        Ok(first)
    }

    /// Validity, or `None` when no geometry is null.
    fn nulls(&self) -> Option<NullBuffer> {
        self.0
            .iter()
            .any(Option::is_none)
            .then(|| NullBuffer::from_iter(self.0.iter().map(Option::is_some)))
    }
}

/// Interleaved coordinates of one dimension.
struct Coords {
    dim: Dimension,
    values: Vec<f64>,
}

impl Coords {
    fn new(dim: Dimension) -> Self {
        Coords {
            dim,
            values: Vec::new(),
        }
    }

    /// Appends a vertex, with `NaN` for ordinates it lacks.
    fn push(&mut self, vertex: Vertex) {
        let z = vertex.z.unwrap_or(f64::NAN);
        let m = vertex.m.unwrap_or(f64::NAN);
        match self.dim {
            Dimension::XY => self.values.extend([vertex.x, vertex.y]),
            Dimension::XYZ => self.values.extend([vertex.x, vertex.y, z]),
            Dimension::XYM => self.values.extend([vertex.x, vertex.y, m]),
            Dimension::XYZM => self.values.extend([vertex.x, vertex.y, z, m]),
        }
    }

    /// Appends the vertices in `range` and returns how many.
    fn push_path(&mut self, vertices: &VertexAttributes, range: Range<usize>) -> usize {
        let count = range.len();
        for i in range {
            self.push(vertices.get(i).unwrap_or_default());
        }
        count
    }

    /// Appends the vertices in `range` and the first again to close them, returning how many.
    fn push_ring(&mut self, vertices: &VertexAttributes, range: Range<usize>) -> usize {
        let first = range.start;
        let count = self.push_path(vertices, range);
        if count == 0 {
            return 0;
        }
        self.push(vertices.get(first).unwrap_or_default());
        count + 1
    }

    fn finish(self) -> Result<CoordBuffer, ToGeoArrowError> {
        let coords = InterleavedCoordBuffer::try_new(ScalarBuffer::from(self.values), self.dim)?;
        Ok(CoordBuffer::Interleaved(coords))
    }
}

/// Offsets that start at 0 and grow by each pushed length.
struct Offsets(Vec<i32>);

impl Offsets {
    fn new() -> Self {
        Offsets(vec![0])
    }

    fn push(&mut self, len: usize) -> Result<(), ToGeoArrowError> {
        let last = self.0.last().copied().unwrap_or(0);
        let len = i32::try_from(len).map_err(|_| ToGeoArrowError::TooLarge)?;
        self.0
            .push(last.checked_add(len).ok_or(ToGeoArrowError::TooLarge)?);
        Ok(())
    }

    fn finish(self) -> OffsetBuffer<i32> {
        OffsetBuffer::new(ScalarBuffer::from(self.0))
    }
}

impl TryFrom<GeometryColumn<'_, Point>> for PointArray {
    type Error = ToGeoArrowError;

    /// Null and empty points both hold `NaN` coordinates; only nulls are marked invalid.
    fn try_from(column: GeometryColumn<'_, Point>) -> Result<Self, Self::Error> {
        let mut coords = Coords::new(column.dimension(|p| p.0.map(|v| v.description()))?);
        for point in column.0 {
            coords.push(point.and_then(|p| p.0).unwrap_or(Vertex {
                x: f64::NAN,
                y: f64::NAN,
                ..Default::default()
            }));
        }
        Ok(PointArray::try_new(coords.finish()?, column.nulls(), Default::default())?)
    }
}

impl TryFrom<GeometryColumn<'_, MultiPoint>> for MultiPointArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, MultiPoint>) -> Result<Self, Self::Error> {
        let mut coords = Coords::new(column.dimension(|mp| Some(mp.vertices.description()))?);
        let mut geom_offsets = Offsets::new();
        for multi_point in column.0 {
            let len = multi_point.as_ref().map_or(0, |mp| {
                coords.push_path(&mp.vertices, 0..mp.vertices.len())
            });
            geom_offsets.push(len)?;
        }
        Ok(MultiPointArray::try_new(
            coords.finish()?,
            geom_offsets.finish(),
            column.nulls(),
            Default::default(),
        )?)
    }
}

impl TryFrom<GeometryColumn<'_, Polyline>> for MultiLineStringArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, Polyline>) -> Result<Self, Self::Error> {
        let mut coords = Coords::new(column.dimension(|p| Some(p.0.vertices.description()))?);
        let mut geom_offsets = Offsets::new();
        let mut line_offsets = Offsets::new();
        for polyline in column.0 {
            let Some(Polyline(paths)) = polyline else {
                geom_offsets.push(0)?;
                continue;
            };
            for i in 0..paths.path_count() {
                let range = paths.path_range(i).unwrap_or(0..0);
                let len = if paths.is_closed_path(i) {
                    coords.push_ring(&paths.vertices, range)
                } else {
                    coords.push_path(&paths.vertices, range)
                };
                line_offsets.push(len)?;
            }
            geom_offsets.push(paths.path_count())?;
        }
        Ok(MultiLineStringArray::try_new(
            coords.finish()?,
            geom_offsets.finish(),
            line_offsets.finish(),
            column.nulls(),
            Default::default(),
        )?)
    }
}

impl TryFrom<GeometryColumn<'_, Polygon>> for MultiPolygonArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, Polygon>) -> Result<Self, Self::Error> {
        let mut coords = Coords::new(column.dimension(|p| Some(p.rings.vertices.description()))?);
        let mut geom_offsets = Offsets::new();
        let mut polygon_offsets = Offsets::new();
        let mut ring_offsets = Offsets::new();
        for polygon in column.0 {
            let Some(polygon) = polygon else {
                geom_offsets.push(0)?;
                continue;
            };
            let mut count = 0;
            for rings in polygon.ogc_polygons() {
                polygon_offsets.push(rings.len())?;
                for i in rings {
                    let range = polygon.rings.path_range(i).unwrap_or(0..0);
                    ring_offsets.push(coords.push_ring(&polygon.rings.vertices, range))?;
                }
                count += 1;
            }
            geom_offsets.push(count)?;
        }
        Ok(MultiPolygonArray::try_new(
            coords.finish()?,
            geom_offsets.finish(),
            polygon_offsets.finish(),
            ring_offsets.finish(),
            column.nulls(),
            Default::default(),
        )?)
    }
}

/// Mixed geometries go through the GeoArrow geometry builder by way of geo-traits.
impl TryFrom<GeometryColumn<'_, Geometry>> for GeometryArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, Geometry>) -> Result<Self, Self::Error> {
        let builder = GeometryBuilder::from_nullable_geometries(column.0, GeometryType::new(Default::default()))?;
        Ok(builder.finish())
    }
}

#[cfg(test)]
mod tests;
