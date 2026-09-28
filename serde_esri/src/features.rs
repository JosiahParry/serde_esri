//! `Feature` and `FeatureSet` objects
//!
//! Geometry objects are often accompanied by attributes that describe the point in space.
//! The Esri [`Feature`](https://developers.arcgis.com/documentation/common-data-types/feature-object.htm)  
//! object enables us to represent geometries and attributes alongside each other.
//!
//! The Esri [`FeatureSet`](https://developers.arcgis.com/documentation/common-data-types/featureset-object.htm)
//! object represents a collection of individual features. This is the most common representation that is encountered
//! when working with a Feature Service via its rest API.
use crate::{
    field_type::FieldType, geometry::EsriGeometry, spatial_reference::SpatialReference,
    sqltype::SqlType,
};
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

mod value;

pub use value::EsriValue;
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};

// handy reference
// https://github.com/Esri/arcgis-rest-js/blob/0e410dc16e0dd2961affb09ff7efbfb9b6c4999a/packages/arcgis-rest-request/src/types/feature.ts#L24

/// A single geometry and its attributes
///
/// Note that both geometry and attributes are optional. This is because
/// we can anticipate receiving _only_ geometries, or _only_ attributes
/// or both together.
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Feature<const N: usize> {
    pub geometry: Option<EsriGeometry<N>>,
    pub attributes: Option<IndexMap<String, EsriValue>>,
}

/// A set of geometries and their attributes
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureSet<const N: usize> {
    pub object_id_field_name: Option<String>,
    pub global_id_field_name: Option<String>,
    pub display_field_name: Option<String>,
    pub geometry_type: Option<String>, // TODO should this be an enum?
    pub spatial_reference: Option<SpatialReference>,
    pub has_z: Option<bool>,
    pub has_m: Option<bool>,
    pub fields: Option<Vec<Field>>,
    pub features: Vec<Feature<N>>,
}

/// Metadata about an attribute field
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    #[serde_as(as = "DisplayFromStr")]
    pub field_type: FieldType,
    pub alias: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub sql_type: Option<SqlType>,
    // unsure what this should be
    pub domain: Option<serde_json::Value>,
    // unsure what this should be
    pub default_value: Option<serde_json::Value>,
}

// using this query for reference
// https://services.arcgis.com/P3ePLMYs2RVChkJx/ArcGIS/rest/services/USA_Counties_Generalized_Boundaries/FeatureServer/0/query?where=1%3D1&objectIds=&time=&geometry=&geometryType=esriGeometryEnvelope&inSR=&spatialRel=esriSpatialRelIntersects&resultType=none&distance=0.0&units=esriSRUnit_Meter&relationParam=&returnGeodetic=false&outFields=*&returnGeometry=true&returnCentroid=false&returnEnvelope=false&featureEncoding=esriDefault&multipatchOption=xyFootprint&maxAllowableOffset=&geometryPrecision=&outSR=&defaultSR=&datumTransformation=&applyVCSProjection=false&returnIdsOnly=false&returnUniqueIdsOnly=false&returnCountOnly=false&returnExtentOnly=false&returnQueryGeometry=false&returnDistinctValues=false&cacheHint=false&orderByFields=&groupByFieldsForStatistics=&outStatistics=&having=&resultOffset=&resultRecordCount=1&returnZ=false&returnM=false&returnExceededLimitFeatures=true&quantizationParameters=&sqlFormat=none&f=pjson&token=
