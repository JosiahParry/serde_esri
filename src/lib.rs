#![doc = include_str!("../README.md")]
//!
//! Example usage:
//!
//! ```rust
//! use serde_esri::features::FeatureSet;
//! use reqwest::Error;
//! use std::io::Read;
//!
//! fn main() {
//!
//!     // USA counties query
//!     let flayer_url = "https://services.arcgis.com/P3ePLMYs2RVChkJx/ArcGIS/rest/services/USA_Counties_Generalized_Boundaries/FeatureServer/0/query?where=1%3D1&outFields=*&returnGeometry=false&resultRecordCount=1&f=json";
//!
//!     // perform request
//!     let mut res = reqwest::blocking::get(flayer_url).unwrap();
//!     let mut body = String::new();
//!
//!     // read body into string
//!     res.read_to_string(&mut body).unwrap();
//!     
//!     // process into FeatureSet
//!     let fset: FeatureSet<2> = serde_json::from_str(&body).unwrap();
//!     println!("{:#?}", fset);

//! }
//! ```
mod de_array;
pub mod features;
pub mod field_type;
pub mod geometry;
pub mod spatial_reference;

// feature flag: geo-types
#[cfg(feature = "geo")]
pub mod geo_types;

#[cfg(feature = "geoarrow")]
pub mod arrow_compat;

#[cfg(feature = "geoarrow")]
mod geoarrow_compat;
