//! Representation of the [Places Service REST API](https://developers.arcgis.com/rest/places/) types and responses. Activate the `"places-client"` feature to enable the `PlacesClient` struct and the ability to query the API.
mod places;
pub use places::*;

pub mod query;
