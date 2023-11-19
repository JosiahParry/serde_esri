mod de_array;
pub mod features;
pub mod field_type;
pub mod geometry;
pub mod spatial_reference;

// feature flag: geo-types
#[cfg(feature = "geo")]
/// Optional feature for converting Esri JSON objects into `geo-types` geometries.
///
/// Z and M dimensions are dropped if present.
pub mod geo_types;
