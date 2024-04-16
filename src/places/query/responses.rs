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
