use crate::{geometry::EsriGeometry, spatial_reference::SpatialReference};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// handy reference
// https://github.com/Esri/arcgis-rest-js/blob/0e410dc16e0dd2961affb09ff7efbfb9b6c4999a/packages/arcgis-rest-request/src/types/feature.ts#L24

#[derive(Debug, Deserialize, Serialize)]
pub struct Feature<const N: usize> {
    pub geometry: Option<EsriGeometry<N>>,
    pub attributes: Option<Map<String, Value>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct FeatureSet<const N: usize> {
    pub objectIdFieldName: Option<String>,
    pub globalIdFieldName: Option<String>,
    pub displayFieldName: Option<String>,
    pub geometryType: Option<String>,
    pub features: Vec<Feature<N>>,
    pub fields: Option<Vec<Field>>,
    pub spatialReference: SpatialReference,
}

// esripbf has most of these defined via Prost
// TODO sqlType, field_type need to be Enums
#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub alias: String,
    pub sqlType: String,
    // unsure what this should be
    pub domain: serde_json::Value,
    // unsure what this should be
    pub defaultValue: serde_json::Value,
}

// using this query for reference
// https://services.arcgis.com/P3ePLMYs2RVChkJx/ArcGIS/rest/services/USA_Counties_Generalized_Boundaries/FeatureServer/0/query?where=1%3D1&objectIds=&time=&geometry=&geometryType=esriGeometryEnvelope&inSR=&spatialRel=esriSpatialRelIntersects&resultType=none&distance=0.0&units=esriSRUnit_Meter&relationParam=&returnGeodetic=false&outFields=*&returnGeometry=true&returnCentroid=false&returnEnvelope=false&featureEncoding=esriDefault&multipatchOption=xyFootprint&maxAllowableOffset=&geometryPrecision=&outSR=&defaultSR=&datumTransformation=&applyVCSProjection=false&returnIdsOnly=false&returnUniqueIdsOnly=false&returnCountOnly=false&returnExtentOnly=false&returnQueryGeometry=false&returnDistinctValues=false&cacheHint=false&orderByFields=&groupByFieldsForStatistics=&outStatistics=&having=&resultOffset=&resultRecordCount=1&returnZ=false&returnM=false&returnExceededLimitFeatures=true&quantizationParameters=&sqlFormat=none&f=pjson&token=
