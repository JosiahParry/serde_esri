//! Represents a spatial reference
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Read more on [Esri docs site](https://developers.arcgis.com/documentation/common-data-types/geometry-objects.htm#GUID-DFF0E738-5A42-40BC-A811-ACCB5814BABC)
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpatialReference {
    pub wkid: Option<u32>,
    pub latest_wkid: Option<u32>,
    pub vcs_wkid: Option<u32>,
    pub latest_vcs_wkid: Option<u32>,
    pub wkt: Option<String>,
}
