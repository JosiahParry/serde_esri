//! `Feature` and `FeatureSet` objects
//!
//! Geometry objects are often accompanied by attributes that describe the point in space.
//! The Esri [`Feature`](https://developers.arcgis.com/documentation/common-data-types/feature-object.htm)  
//! object enables us to represent geometries and attributes alongside each other.
//!
//! The Esri [`FeatureSet`](https://developers.arcgis.com/documentation/common-data-types/featureset-object.htm)
//! object represents a collection of individual features. This is the most common representation that is encountered
//! when working with a Feature Service via its rest API.
use crate::{field_type::FieldType, geometry::EsriGeometry, spatial_reference::SpatialReference};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::{serde_as, DisplayFromStr};

// handy reference
// https://github.com/Esri/arcgis-rest-js/blob/0e410dc16e0dd2961affb09ff7efbfb9b6c4999a/packages/arcgis-rest-request/src/types/feature.ts#L24

/// A single geometry and its attributes
///
/// Note that both geometry and attributes are optional. This is because
/// we can anticipate receiving _only_ geometries, or _only_ attributes
/// or both together.
#[derive(Debug, Deserialize, Serialize)]
pub struct Feature<const N: usize> {
    pub geometry: Option<EsriGeometry<N>>,
    pub attributes: Option<Map<String, Value>>,
}

/// A set of geometries and their attributes
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize)]
pub struct FeatureSet<const N: usize> {
    pub objectIdFieldName: Option<String>,
    pub globalIdFieldName: Option<String>,
    pub displayFieldName: Option<String>,
    pub spatialReference: Option<SpatialReference>,
    pub geometryType: Option<String>, // TODO should this be an enum?
    pub features: Vec<Feature<N>>,
    pub fields: Option<Vec<Field>>,
}

/// Metadata about an attribute field
// esripbf has most of these defined via Prost
// TODO sqlType, field_type need to be Enums
#[serde_as]
#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    #[serde_as(as = "DisplayFromStr")]
    pub field_type: FieldType,
    pub alias: Option<String>,
    pub sqlType: Option<String>,
    // unsure what this should be
    pub domain: Option<serde_json::Value>,
    // unsure what this should be
    pub defaultValue: Option<serde_json::Value>,
}

// using this query for reference
// https://services.arcgis.com/P3ePLMYs2RVChkJx/ArcGIS/rest/services/USA_Counties_Generalized_Boundaries/FeatureServer/0/query?where=1%3D1&objectIds=&time=&geometry=&geometryType=esriGeometryEnvelope&inSR=&spatialRel=esriSpatialRelIntersects&resultType=none&distance=0.0&units=esriSRUnit_Meter&relationParam=&returnGeodetic=false&outFields=*&returnGeometry=true&returnCentroid=false&returnEnvelope=false&featureEncoding=esriDefault&multipatchOption=xyFootprint&maxAllowableOffset=&geometryPrecision=&outSR=&defaultSR=&datumTransformation=&applyVCSProjection=false&returnIdsOnly=false&returnUniqueIdsOnly=false&returnCountOnly=false&returnExtentOnly=false&returnQueryGeometry=false&returnDistinctValues=false&cacheHint=false&orderByFields=&groupByFieldsForStatistics=&outStatistics=&having=&resultOffset=&resultRecordCount=1&returnZ=false&returnM=false&returnExceededLimitFeatures=true&quantizationParameters=&sqlFormat=none&f=pjson&token=
