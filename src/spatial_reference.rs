use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpatialReference {
    pub wkid: Option<u32>,
    pub latest_wkid: Option<u32>,
    pub vcs_wkid: Option<u32>,
    pub latest_vcs_wkid: Option<u32>,
    pub wkt: Option<String>,
}
