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

pub mod description;
mod envelope;
pub mod flags;
#[cfg(feature = "geo-traits")]
pub mod from_geo_traits;
pub mod from_shape;
pub mod geometry;
#[cfg(feature = "geo-traits")]
#[macro_use]
pub mod geo_traits_impl;
mod rings;
pub mod to_shape;
#[cfg(feature = "geoarrow")]
pub mod to_geoarrow;
pub mod shape;
pub mod vertex;

#[cfg(test)]
mod tests;

