//! A client for the ArcGIS Places API.
//!
//! The [`PlacesClient`] struct is used to interact with the API. You can create a new client with the [`PlacesClient::new()`] method.
//!
//! Each endpoint is supported from the client, and you can query them with the appropriate method. For example, to query the [`/places/near-point`](https://developers.arcgis.com/rest/places/near-point-get/) endpoint, you can use the [`PlacesClient::near_point()`] method.
//!
//! The client is responsible for handling the authorization token and making the requests to the API.
//!
//! For [`PlacesClient::near_point()`] and [`PlacesClient::within_extent()`] queries, the client will automatically handle pagination and return an iterator over the results. This iterator will fetch the next page when needed. It will not, however, allow you to modify the page size. The default will be used.

mod query_params;
mod responses;
pub use query_params::*;
pub use responses::*;

#[cfg(feature = "places-client")]
mod client;
#[cfg(feature = "places-client")]
mod place_search;

#[cfg(feature = "places-client")]
pub use client::*;
#[cfg(feature = "places-client")]
pub use place_search::*;
