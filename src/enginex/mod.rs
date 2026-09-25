//! Pure Rust representation of the in-memory geometry model used by the
//! [Esri Geometry API for Java](https://github.com/Esri/geometry-api-java).
//!
//! Types mirror the engine's storage so its data can be used directly:
//!
//! - Vertices are stored column-wise, one buffer per vertex attribute
//!   (`MultiVertexGeometryImpl.m_vertexAttributes`). The xy buffer is
//!   interleaved, so `Vec<[f64; 2]>` has the same layout as the engine's
//!   `double[]`.
//! - Multipaths partition that vertex buffer into paths with `path_offsets`
//!   and store per-path and per-segment flags (`MultiPathImpl`).
//! - Closed paths do not repeat their first vertex; the closing segment is
//!   implicit.
//! - Non-linear segments keep their parameters in a shared `f64` buffer,
//!   addressed per vertex by an index where `-1` means no parameters.
//!
//! Supported vertex attributes are x/y, z, m, and id.
//!
//! Read geometries from Esri shape buffers with [`shape::EsriShape`], or convert shapefile
//! records with `Geometry::try_from(crate::shape::Shape)`.

mod description;
mod flags;
mod from_shape;
mod geometry;
#[cfg(feature = "geo-traits")]
mod geo_traits_impl;
mod rings;
#[cfg(feature = "geoarrow-array")]
mod to_geoarrow;
pub mod shape;
mod vertex;

#[cfg(test)]
mod tests;

pub use description::{Semantics, VertexDescription};
pub use flags::{FillRule, GeometryType, PathFlag, PathFlags, SegmentFlags, SegmentType};
pub use from_shape::FromShapeError;
pub use geometry::{
    Envelope, Envelope2D, Geometry, Interval, Line, MultiPath, MultiPoint, Point, Polygon, Polyline,
    Segments,
};
pub use vertex::{Vertex, VertexAttributes};
#[cfg(feature = "geo-traits")]
pub use geo_traits_impl::{PathView, PolygonView};
#[cfg(feature = "geoarrow-array")]
pub use to_geoarrow::{GeometryColumn, ToGeoArrowError};
