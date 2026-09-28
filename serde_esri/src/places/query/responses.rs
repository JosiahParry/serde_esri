use crate::places::Pagination;
use crate::places::{CategoryDetails, PlaceDetails, PlaceResult};
use serde::{Deserialize, Serialize};

/// Represent the response from the /places/{placeId} endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceResponse {
    #[serde(rename = "placeDetails")]
    pub place_details: PlaceDetails,
}

/// Represents the response from the /categories endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriesResponse {
    pub categories: Vec<CategoryDetails>,
}

/// Represents the response from the /places/near-point and /places/within-extent endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointResponse {
    pub results: Vec<PlaceResult>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorDetails {
    pub code: u16,
    pub message: String,
    pub details: Vec<String>,
    #[serde(rename = "restInfoUrl")]
    pub rest_info_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ExpectedResponse {
    Point(PointResponse),
    Error(ErrorResponse),
}

#[cfg(feature = "places-client")]
#[derive(Debug)]
pub enum PlacesError {
    RequestError(reqwest::Error),
    ApiError(ErrorResponse),
}
