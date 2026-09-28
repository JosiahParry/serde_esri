//! Represents a spatial reference
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Read more on [Esri docs site](https://developers.arcgis.com/documentation/common-data-types/geometry-objects.htm#GUID-DFF0E738-5A42-40BC-A811-ACCB5814BABC)
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpatialReference {
    pub wkid: Option<i32>,
    pub latest_wkid: Option<i32>,
    pub vcs_wkid: Option<i32>,
    pub latest_vcs_wkid: Option<i32>,
    pub wkt: Option<String>,
    pub wkt2: Option<String>,
}

impl Default for SpatialReference {
    fn default() -> Self {
        Self {
            wkid: Some(3857_i32),
            latest_wkid: None,
            vcs_wkid: None,
            latest_vcs_wkid: None,
            wkt: None,
            wkt2: None,
        }
    }
}

#[cfg(test)]
mod tests;
