//! Converts engine geometries into GeoArrow arrays with interleaved coordinates, pushing their
//! geo-traits views through geoarrow-array's builders. Vertex IDs are dropped.
//!
//! ```ignore
//! let polygons: Vec<Option<Polygon>> = ...;
//! let array = MultiPolygonArray::try_from(GeometryColumn(&polygons))?;
//! ```

use crate::enginex::{
    Attribute, Envelope, Geometry, Interval, MultiPoint, Point, Polygon, Polyline,
    VertexDescription,
};
use arrow_buffer::{NullBuffer, ScalarBuffer};
use geoarrow_array::{
    array::{
        GeometryArray, MultiLineStringArray, MultiPointArray, MultiPolygonArray, PointArray,
        RectArray, SeparatedCoordBuffer,
    },
    builder::{
        GeometryBuilder, MultiLineStringBuilder, MultiPointBuilder, MultiPolygonBuilder,
        PointBuilder,
    },
    capacity::{MultiLineStringCapacity, MultiPointCapacity, MultiPolygonCapacity},
};
use geoarrow_schema::{
    error::GeoArrowError, CoordType, Dimension, GeometryType, MultiLineStringType,
    MultiPointType, MultiPolygonType, PointType,
};

/// Engine geometries of one kind, with `None` for nulls, to convert into a GeoArrow array.
#[derive(Clone, Copy, Debug)]
pub struct GeometryColumn<'a, G>(pub &'a [Option<G>]);

#[derive(Debug)]
pub enum ToGeoArrowError {
    /// Geometries in the column carry different Z and M attributes.
    MixedDimensions,
    GeoArrow(GeoArrowError),
}

impl std::fmt::Display for ToGeoArrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToGeoArrowError::MixedDimensions => {
                write!(f, "geometries in a column must share their dimensions")
            }
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
        match (description.has(Attribute::Z), description.has(Attribute::M)) {
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

/// Null and empty points both hold `NaN` coordinates; only nulls are marked invalid.
impl TryFrom<GeometryColumn<'_, Point>> for PointArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, Point>) -> Result<Self, Self::Error> {
        let dim = column.dimension(|p| p.0.map(|v| v.description()))?;
        let typ = PointType::new(dim, Default::default()).with_coord_type(CoordType::Interleaved);
        let mut builder = PointBuilder::with_capacity(typ, column.0.len());
        for point in column.0 {
            builder.try_push_point(point.as_ref())?;
        }
        Ok(builder.finish())
    }
}

impl TryFrom<GeometryColumn<'_, MultiPoint>> for MultiPointArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, MultiPoint>) -> Result<Self, Self::Error> {
        let dim = column.dimension(|mp| Some(mp.vertices.description()))?;
        let typ =
            MultiPointType::new(dim, Default::default()).with_coord_type(CoordType::Interleaved);
        let capacity = MultiPointCapacity::from_multi_points(column.0.iter().map(Option::as_ref));
        let mut builder = MultiPointBuilder::with_capacity(typ, capacity);
        for multi_point in column.0 {
            builder.push_multi_point(multi_point.as_ref())?;
        }
        Ok(builder.finish())
    }
}

/// Closed paths repeat their first vertex.
impl TryFrom<GeometryColumn<'_, Polyline>> for MultiLineStringArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, Polyline>) -> Result<Self, Self::Error> {
        let dim = column.dimension(|p| Some(p.0.vertices.description()))?;
        let typ = MultiLineStringType::new(dim, Default::default())
            .with_coord_type(CoordType::Interleaved);
        let capacity =
            MultiLineStringCapacity::from_multi_line_strings(column.0.iter().map(Option::as_ref));
        let mut builder = MultiLineStringBuilder::with_capacity(typ, capacity);
        for polyline in column.0 {
            builder.push_multi_line_string(polyline.as_ref())?;
        }
        Ok(builder.finish())
    }
}

/// Envelopes as GeoArrow boxes, with Z and M ranges as their own dimensions. `RectBuilder` only
/// pushes infallibly, and envelopes may lack the column's Z or M, so the buffers are built here.
impl TryFrom<GeometryColumn<'_, Envelope>> for RectArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, Envelope>) -> Result<Self, Self::Error> {
        let dim = column.dimension(|e| {
            let attributes = [e.z.map(|_| Attribute::Z), e.m.map(|_| Attribute::M)];
            e.xy.map(|_| attributes.into_iter().flatten().collect())
        })?;
        let size = match dim {
            Dimension::XY => 2,
            Dimension::XYZ | Dimension::XYM => 3,
            Dimension::XYZM => 4,
        };
        let mut lower = vec![Vec::with_capacity(column.0.len()); size];
        let mut upper = vec![Vec::with_capacity(column.0.len()); size];
        for envelope in column.0 {
            let envelope = envelope.unwrap_or_default();
            let xy = envelope.xy.map_or([f64::NAN; 4], |e| [e.xmin, e.ymin, e.xmax, e.ymax]);
            let range = |i: Option<Interval<f64>>| i.map_or([f64::NAN; 2], |i| [i.min, i.max]);
            let [zmin, zmax] = range(envelope.z);
            let [mmin, mmax] = range(envelope.m);
            let (mins, maxes) = match dim {
                Dimension::XY => (vec![xy[0], xy[1]], vec![xy[2], xy[3]]),
                Dimension::XYZ => (vec![xy[0], xy[1], zmin], vec![xy[2], xy[3], zmax]),
                Dimension::XYM => (vec![xy[0], xy[1], mmin], vec![xy[2], xy[3], mmax]),
                Dimension::XYZM => (vec![xy[0], xy[1], zmin, mmin], vec![xy[2], xy[3], zmax, mmax]),
            };
            for (buffer, value) in lower.iter_mut().zip(mins) {
                buffer.push(value);
            }
            for (buffer, value) in upper.iter_mut().zip(maxes) {
                buffer.push(value);
            }
        }
        let buffers = |values: Vec<Vec<f64>>| {
            SeparatedCoordBuffer::from_vec(values.into_iter().map(ScalarBuffer::from).collect(), dim)
        };
        Ok(RectArray::new(
            buffers(lower)?,
            buffers(upper)?,
            column.nulls(),
            Default::default(),
        ))
    }
}

/// Rings are grouped into polygons by orientation (see [`Polygon::ogc_polygons`]) and closed.
impl TryFrom<GeometryColumn<'_, Polygon>> for MultiPolygonArray {
    type Error = ToGeoArrowError;

    fn try_from(column: GeometryColumn<'_, Polygon>) -> Result<Self, Self::Error> {
        let dim = column.dimension(|p| Some(p.rings.vertices.description()))?;
        let typ =
            MultiPolygonType::new(dim, Default::default()).with_coord_type(CoordType::Interleaved);
        let capacity =
            MultiPolygonCapacity::from_multi_polygons(column.0.iter().map(Option::as_ref));
        let mut builder = MultiPolygonBuilder::with_capacity(typ, capacity);
        for polygon in column.0 {
            builder.push_multi_polygon(polygon.as_ref())?;
        }
        Ok(builder.finish())
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
