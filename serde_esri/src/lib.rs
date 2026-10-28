#![doc = include_str!("../README.md")]

mod de_array;
#[macro_use]
pub mod enginex;
pub mod features;
pub mod field_type;
pub mod geometry;
pub mod places;
pub mod shape;
pub mod spatial_reference;
pub mod sqltype;
// feature flag: geo-types
#[cfg(feature = "geo")]
pub mod geo_types;

#[cfg(feature = "geoarrow")]
pub mod arrow_compat;

#[cfg(feature = "from-geo")]
#[allow(clippy::from_over_into)]
pub mod geo;
