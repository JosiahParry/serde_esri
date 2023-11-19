mod de_array;
pub mod features;
pub mod field_type;
pub mod geometry;
pub mod spatial_reference;

// feature flag: geo-types
#[cfg(feature = "geo")]
pub mod geo_types;
