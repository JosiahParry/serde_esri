#![doc = include_str!("../README.md")]

mod de_array;
pub mod features;
pub mod field_type;
pub mod geometry;
pub mod places;
pub mod spatial_reference;
// feature flag: geo-types
#[cfg(feature = "geo")]
pub mod geo_types;

#[cfg(feature = "geoarrow")]
pub mod arrow_compat;

#[cfg(feature = "geoarrow")]
mod geoarrow_compat;
