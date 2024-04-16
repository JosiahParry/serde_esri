use crate::places::query::{
    CategoriesQueryParams, CategoriesResponse, CategoryQueryParams, NearPointQuery,
    NearPointQueryParams, PlaceQueryParams, PlaceResponse, WithinExtentQuery,
    WithinExtentQueryParams,
};
use crate::places::CategoryDetails;
use reqwest::Result;
use std::sync::Arc;

/// The base URL for the Places API
pub const PLACES_API_URL: &str =
    "https://places-api.arcgis.com/arcgis/rest/services/places-service/v1";

/// A client for the ArcGIS Places API
///
/// ```
/// use crate::places::PlacesClient;
///
/// let client = PlacesClient::new(PLACES_API_URL, "your token");
/// ```
///
/// ### Note
///
/// Replace `"your token"` with your actual token.
///
#[derive(Debug, Clone)]
pub struct PlacesClient {
    pub base_url: String,
    pub(crate) client: reqwest::blocking::Client,
    /// The token to use for authorization.
    pub(crate) token: String,
}

impl PlacesClient {
    /// Create a new client for the Places API
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::blocking::Client::new(),
            token: token.to_string(),
        }
    }

    /// Query the [`/places/near-point`](https://developers.arcgis.com/rest/places/near-point-get/) endpoint
    pub fn near_point(&self, params: NearPointQueryParams) -> Result<NearPointQuery> {
        NearPointQuery::new(Arc::new(self.clone()), params)
    }

    pub fn within_extent(&self, params: WithinExtentQueryParams) -> Result<WithinExtentQuery> {
        WithinExtentQuery::new(Arc::new(self.clone()), params)
    }

    /// Query the [`/places/{place_id}`](https://developers.arcgis.com/rest/places/place-details-get/) endpoint
    pub fn place_details(&self, params: PlaceQueryParams) -> Result<PlaceResponse> {
        let fields = params.requested_fields.join(",");

        let c = self
            .client
            .get(format!("{}/places/{}", self.base_url, params.place_id))
            .header(
                "X-Esri-Authorization",
                format!("Bearer {}", self.token.as_str()),
            )
            .query(&vec![("requestedFields", fields.as_str())])
            .send()?
            .json::<PlaceResponse>()?;

        // TODO: handle errors make custom serde error types
        Ok(c)
    }

    /// Query the [`/categories`](https://developers.arcgis.com/rest/places/categories-get/) endpoint
    pub fn categories(&self, params: CategoriesQueryParams) -> Result<CategoriesResponse> {
        let c = self
            .client
            .get(format!("{}/categories", self.base_url))
            .header(
                "X-Esri-Authorization",
                format!("Bearer {}", self.token.as_str()),
            )
            .query(&params)
            .send()?
            .json::<CategoriesResponse>()?;
        Ok(c)
    }

    /// Query the [`/categories/{categoryId}`](https://developers.arcgis.com/rest/places/category-details-get/) endpoint
    pub fn category_details(&self, params: CategoryQueryParams) -> Result<CategoryDetails> {
        let c = self
            .client
            .get(format!(
                "{}/categories/{}",
                self.base_url, params.category_id
            ))
            .header(
                "X-Esri-Authorization",
                format!("Bearer {}", self.token.as_str()),
            )
            .query(&params)
            .send()?
            .json::<CategoryDetails>()?;
        Ok(c)
    }
}
