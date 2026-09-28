// TODO:
// create place_search.rs for managing pagination from /places/near-point and /places/within-extent
// create mod.rs
// move to serde_esri
// feature gate Client and NearbyQuery as well as WithinQuery

use crate::places::{
    query::{
        ExpectedResponse, NearPointQueryParams, PlacesClient, PlacesError, PointResponse,
        WithinExtentQueryParams,
    },
    PlaceResult,
};
use std::sync::Arc;

/// Struct used to query the /places/near-point endpoint
#[derive(Debug, Clone)]
pub struct NearPointQuery {
    /// The client as created by [`PlacesClient::new()`]
    pub client: Arc<PlacesClient>,
    /// The parameters used to query the endpoint
    pub params: NearPointQueryParams,
    /// The results of the query as an iterator. This iterator will automatically fetch the next page when needed.
    pub results: <Vec<PlaceResult> as IntoIterator>::IntoIter,
    /// The next page to fetch. This is automatically updated when the iterator reaches the end of the current page.
    pub next_page: Option<String>,
}

impl NearPointQuery {
    /// Create a new [`NearPointQuery`] from a [`PlacesClient`] and a [`NearPointQueryParams`]
    /// This will send the initial request and parse the response. The next page
    /// is stored in the `next_page` field. Use  `.into_iter()` to iterate over the results
    /// and the subsequent pages.
    ///
    /// Note that requests are paginated so these impls use a blocking reqwest client.
    pub fn new(
        client: Arc<PlacesClient>,
        params: NearPointQueryParams,
    ) -> Result<Self, PlacesError> {
        // create the initial request
        let c = client
            .client
            .get(format!("{}/places/near-point", client.base_url))
            .query(&params.clone().prepare())
            .header("X-Esri-Authorization", format!("Bearer {}", client.token));

        // send the request and parse the response
        let resp = c
            .send()
            .map_err(PlacesError::RequestError)?
            .json::<ExpectedResponse>()
            .map_err(PlacesError::RequestError)?;

        // Handle the ExpectedResponse
        let point_response = match resp {
            ExpectedResponse::Point(point_response) => point_response,
            ExpectedResponse::Error(error_response) => {
                return Err(PlacesError::ApiError(error_response))
            }
        };

        // fetch the pagination
        let next_page = match point_response.pagination {
            Some(p) => p.next_url,
            None => None,
        };

        // return the results
        Ok(Self {
            client,
            params,
            results: point_response.results.into_iter(),
            next_page,
        })
    }

    pub fn try_next(&mut self) -> Result<Option<PlaceResult>, PlacesError> {
        if let Some(place_res) = self.results.next() {
            return Ok(Some(place_res));
        }

        if self.next_page.is_none() {
            return Ok(None);
        }

        let next_page = self
            .client
            .client
            .get(self.next_page.as_ref().unwrap())
            .header(
                "X-Esri-Authorization",
                format!("Bearer {}", self.client.token),
            )
            .send()
            .map_err(PlacesError::RequestError)?
            .json::<PointResponse>()
            .map_err(PlacesError::RequestError)?;

        self.results = next_page.results.into_iter();
        self.next_page = match next_page.pagination {
            Some(p) => p.next_url,
            None => None,
        };

        Ok(self.results.next())
    }
}

/// This lets you paginate through the results of a NearbyQuery
impl Iterator for NearPointQuery {
    type Item = Result<PlaceResult, PlacesError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(place_res)) => Some(Ok(place_res)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Struct used to query the /places/near-point endpoint
#[derive(Debug, Clone)]
pub struct WithinExtentQuery {
    /// The client as created by [`PlacesClient::new()`]
    pub client: Arc<PlacesClient>,
    /// The parameters used to query the endpoint
    pub params: WithinExtentQueryParams,
    /// The results of the query as an iterator. This iterator will automatically fetch the next page when needed.
    pub results: <Vec<PlaceResult> as IntoIterator>::IntoIter,
    /// The next page to fetch. This is automatically updated when the iterator reaches the end of the current page.
    pub next_page: Option<String>,
}
impl WithinExtentQuery {
    pub fn new(
        client: Arc<PlacesClient>,
        params: WithinExtentQueryParams,
    ) -> Result<Self, PlacesError> {
        // create the initial request
        let c = client
            .client
            .get(format!("{}/places/within-extent", client.base_url))
            .query(&params.clone().prepare())
            .header("X-Esri-Authorization", format!("Bearer {}", client.token));

        // send the request and parse the response
        let resp = c
            .send()
            .map_err(PlacesError::RequestError)?
            .json::<ExpectedResponse>()
            .map_err(PlacesError::RequestError)?;

        // Handle the ExpectedResponse
        let point_response = match resp {
            ExpectedResponse::Point(point_response) => point_response,
            ExpectedResponse::Error(error_response) => {
                return Err(PlacesError::ApiError(error_response))
            }
        };

        // fetch the pagination
        let next_page = match point_response.pagination {
            Some(p) => p.next_url,
            None => None,
        };

        // return the results
        Ok(Self {
            client,
            params,
            results: point_response.results.into_iter(),
            next_page,
        })
    }

    pub fn try_next(&mut self) -> Result<Option<PlaceResult>, PlacesError> {
        if let Some(place_res) = self.results.next() {
            return Ok(Some(place_res));
        }

        if self.next_page.is_none() {
            return Ok(None);
        }

        let next_page = self
            .client
            .client
            .get(self.next_page.as_ref().unwrap())
            .header(
                "X-Esri-Authorization",
                format!("Bearer {}", self.client.token),
            )
            .send()
            .map_err(PlacesError::RequestError)?
            .json::<PointResponse>()
            .map_err(PlacesError::RequestError)?;

        self.results = next_page.results.into_iter();
        self.next_page = match next_page.pagination {
            Some(p) => p.next_url,
            None => None,
        };

        Ok(self.results.next())
    }
}

impl Iterator for WithinExtentQuery {
    type Item = Result<PlaceResult, PlacesError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(place_res)) => Some(Ok(place_res)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
