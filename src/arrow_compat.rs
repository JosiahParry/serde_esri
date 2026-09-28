//! Converts FeatureSets into an Arrow [`RecordBatch`]: one column per field and a GeoArrow
//! geometry column.
//!
//! ```ignore
//! // From the JSON a service returns, streamed straight into Arrow; the fastest path.
//! let batch = RecordBatch::try_from(FeatureSetJson(&bytes))?;
//! // From a FeatureSet already parsed, through the engine (see [`crate::enginex`]).
//! let batch = RecordBatch::try_from(&feature_set)?;
//! ```
//!
//! Integers, floats, strings, GUIDs, and XML map to their Arrow types, dates to UTC millisecond
//! timestamps, and geometry fields are left to the geometry column. The geometry column's CRS
//! comes from the spatial reference: WKIDs below 100000 as EPSG codes, others as ESRI codes.

use crate::{
    enginex::{Geometry, GeometryColumn, ToGeoArrowError},
    features::{Feature, FeatureSet, Field},
    field_type::FieldType,
    geometry::FromEsriError,
    spatial_reference::SpatialReference,
};
use arrow_array::{
    ArrayRef, Date32Array, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array,
    LargeStringArray, Time32MillisecondArray,
    RecordBatch, RecordBatchOptions, StringArray, TimestampMillisecondArray,
};
use arrow_schema::{ArrowError, Field as ArrowField, Schema};
use geoarrow_array::{
    array::{GeometryArray, MultiLineStringArray, MultiPointArray, MultiPolygonArray, PointArray},
    GeoArrowArray,
};
use geoarrow_schema::{error::GeoArrowError, Crs, Metadata};
use serde_json::Value;
use std::sync::Arc;

mod columns;
mod geometry;
mod json;
mod temporal;

pub use json::FeatureSetJson;
use temporal::{DateOnly, TimeOnly, TimestampOffset};

#[derive(Debug)]
pub enum ToArrowError {
    /// The field's type has no Arrow column.
    UnsupportedField { name: String, field_type: FieldType },
    /// `geometryType` names no supported geometry type.
    UnsupportedGeometryType(String),
    /// A feature's geometry differs from the feature set's `geometryType`.
    GeometryTypeMismatch,
    Geometry(FromEsriError),
    GeoArrow(ToGeoArrowError),
    Arrow(ArrowError),
    /// The input is not valid FeatureSet JSON.
    Json(serde_json::Error),
    /// The service returned an error object instead of a FeatureSet.
    Service {
        code: Option<i64>,
        message: String,
    },
}

impl std::fmt::Display for ToArrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToArrowError::UnsupportedField { name, field_type } => {
                write!(f, "field {name} has unsupported type {field_type}")
            }
            ToArrowError::UnsupportedGeometryType(t) => write!(f, "unsupported geometry type {t}"),
            ToArrowError::GeometryTypeMismatch => {
                write!(f, "a feature's geometry differs from the feature set's geometry type")
            }
            ToArrowError::Geometry(e) => write!(f, "{e}"),
            ToArrowError::GeoArrow(e) => write!(f, "{e}"),
            ToArrowError::Arrow(e) => write!(f, "{e}"),
            ToArrowError::Json(e) => write!(f, "{e}"),
            ToArrowError::Service { code, message } => match code {
                Some(code) => write!(f, "service error {code}: {message}"),
                None => write!(f, "service error: {message}"),
            },
        }
    }
}

impl std::error::Error for ToArrowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ToArrowError::Geometry(e) => Some(e),
            ToArrowError::GeoArrow(e) => Some(e),
            ToArrowError::Arrow(e) => Some(e),
            ToArrowError::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<FromEsriError> for ToArrowError {
    fn from(e: FromEsriError) -> Self {
        ToArrowError::Geometry(e)
    }
}

impl From<ToGeoArrowError> for ToArrowError {
    fn from(e: ToGeoArrowError) -> Self {
        ToArrowError::GeoArrow(e)
    }
}

impl From<ArrowError> for ToArrowError {
    fn from(e: ArrowError) -> Self {
        ToArrowError::Arrow(e)
    }
}

impl From<GeoArrowError> for ToArrowError {
    fn from(e: GeoArrowError) -> Self {
        ToArrowError::GeoArrow(e.into())
    }
}

impl From<serde_json::Error> for ToArrowError {
    fn from(e: serde_json::Error) -> Self {
        ToArrowError::Json(e)
    }
}

impl Field {
    /// The field's values across `features`, or `None` for geometry fields.
    fn column<const N: usize>(
        &self,
        features: &[Feature<N>],
    ) -> Result<Option<ArrayRef>, ToArrowError> {
        let values = features
            .iter()
            .map(|f| f.attributes.as_ref().and_then(|a| a.get(&self.name)));
        let integers = values.clone().map(|v| v.and_then(Value::as_i64));
        let floats = values.clone().map(|v| v.and_then(Value::as_f64));
        let strings = values.map(|v| v.and_then(Value::as_str));
        let array: ArrayRef = match self.field_type {
            FieldType::EsriFieldTypeSmallInteger => Arc::new(
                integers
                    .map(|v| v.and_then(|v| i16::try_from(v).ok()))
                    .collect::<Int16Array>(),
            ),
            FieldType::EsriFieldTypeInteger => Arc::new(
                integers
                    .map(|v| v.and_then(|v| i32::try_from(v).ok()))
                    .collect::<Int32Array>(),
            ),
            FieldType::EsriFieldTypeOid | FieldType::EsriFieldTypeBigInteger => {
                Arc::new(integers.collect::<Int64Array>())
            }
            FieldType::EsriFieldTypeDateOnly => Arc::new(
                strings
                    .map(|v| v.and_then(|v| v.parse().ok()).map(|DateOnly(days)| days))
                    .collect::<Date32Array>(),
            ),
            FieldType::EsriFieldTypeTimeOnly => Arc::new(
                strings
                    .map(|v| v.and_then(|v| v.parse().ok()).map(|TimeOnly(ms)| ms))
                    .collect::<Time32MillisecondArray>(),
            ),
            FieldType::EsriFieldTypeTimestampOffset => Arc::new(
                strings
                    .map(|v| v.and_then(|v| v.parse().ok()).map(|TimestampOffset(ms)| ms))
                    .collect::<TimestampMillisecondArray>()
                    .with_timezone("UTC"),
            ),
            FieldType::EsriFieldTypeSingle => {
                Arc::new(floats.map(|v| v.map(|v| v as f32)).collect::<Float32Array>())
            }
            FieldType::EsriFieldTypeDouble => Arc::new(floats.collect::<Float64Array>()),
            FieldType::EsriFieldTypeString
            | FieldType::EsriFieldTypeGuid
            | FieldType::EsriFieldTypeGlobalId => Arc::new(strings.collect::<StringArray>()),
            FieldType::EsriFieldTypeXml => Arc::new(strings.collect::<LargeStringArray>()),
            FieldType::EsriFieldTypeDate => Arc::new(
                integers
                    .collect::<TimestampMillisecondArray>()
                    .with_timezone("UTC"),
            ),
            FieldType::EsriFieldTypeGeometry => return Ok(None),
            FieldType::EsriFieldTypeBlob | FieldType::EsriFieldTypeRaster => {
                return Err(ToArrowError::UnsupportedField {
                    name: self.name.clone(),
                    field_type: self.field_type.clone(),
                })
            }
        };
        Ok(Some(array))
    }
}

/// The latest WKID, else the WKID, as an EPSG code below 100000 and an ESRI code otherwise;
/// without either, the WKT2 or WKT.
impl From<&SpatialReference> for Crs {
    fn from(sr: &SpatialReference) -> Self {
        match (sr.latest_wkid.or(sr.wkid), &sr.wkt2, &sr.wkt) {
            (Some(wkid), ..) if wkid < 100_000 => Crs::from_authority_code(format!("EPSG:{wkid}")),
            (Some(wkid), ..) => Crs::from_authority_code(format!("ESRI:{wkid}")),
            (None, Some(wkt2), _) => Crs::from_wkt2_2019(wkt2.clone()),
            (None, None, Some(wkt)) => Crs::from_unknown_crs_type(wkt.clone()),
            (None, None, None) => Crs::default(),
        }
    }
}

impl<const N: usize> FeatureSet<N> {
    /// The geometry field and column as the GeoArrow array `geometryType` calls for.
    fn geometry_column(
        &self,
        geometry_type: &str,
        metadata: Arc<Metadata>,
    ) -> Result<(ArrowField, ArrayRef), ToArrowError> {
        fn typed<T>(
            geometries: &[Option<Geometry>],
            pick: fn(Geometry) -> Option<T>,
        ) -> Result<Vec<Option<T>>, ToArrowError> {
            geometries
                .iter()
                .cloned()
                .map(|g| g.map(|g| pick(g).ok_or(ToArrowError::GeometryTypeMismatch)).transpose())
                .collect()
        }
        fn column(array: impl GeoArrowArray) -> (ArrowField, ArrayRef) {
            (array.data_type().to_field("geometry", true), array.to_array_ref())
        }

        let geometries = self
            .features
            .iter()
            .map(|f| f.geometry.as_ref().map(Geometry::try_from).transpose())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(match geometry_type {
            "esriGeometryPoint" => {
                let points = typed(&geometries, |g| match g {
                    Geometry::Point(p) => Some(p),
                    _ => None,
                })?;
                column(PointArray::try_from(GeometryColumn(&points))?.with_metadata(metadata))
            }
            "esriGeometryMultipoint" => {
                let multi_points = typed(&geometries, |g| match g {
                    Geometry::MultiPoint(mp) => Some(mp),
                    _ => None,
                })?;
                let array = MultiPointArray::try_from(GeometryColumn(&multi_points))?;
                column(array.with_metadata(metadata))
            }
            "esriGeometryPolyline" => {
                let polylines = typed(&geometries, |g| match g {
                    Geometry::Polyline(p) => Some(p),
                    _ => None,
                })?;
                let array = MultiLineStringArray::try_from(GeometryColumn(&polylines))?;
                column(array.with_metadata(metadata))
            }
            "esriGeometryPolygon" => {
                let polygons = typed(&geometries, |g| match g {
                    Geometry::Polygon(p) => Some(p),
                    _ => None,
                })?;
                let array = MultiPolygonArray::try_from(GeometryColumn(&polygons))?;
                column(array.with_metadata(metadata))
            }
            "esriGeometryEnvelope" => {
                let array = GeometryArray::try_from(GeometryColumn(&geometries))?;
                column(array.with_metadata(metadata))
            }
            other => return Err(ToArrowError::UnsupportedGeometryType(other.to_string())),
        })
    }
}

impl<const N: usize> TryFrom<&FeatureSet<N>> for RecordBatch {
    type Error = ToArrowError;

    fn try_from(feature_set: &FeatureSet<N>) -> Result<Self, Self::Error> {
        let mut fields = Vec::new();
        let mut columns = Vec::new();
        for field in feature_set.fields.iter().flatten() {
            if let Some(column) = field.column(&feature_set.features)? {
                fields.push(ArrowField::new(&field.name, column.data_type().clone(), true));
                columns.push(column);
            }
        }

        if let Some(geometry_type) = &feature_set.geometryType {
            let crs = feature_set
                .spatialReference
                .as_ref()
                .map(Crs::from)
                .unwrap_or_default();
            let metadata = Arc::new(Metadata::new(crs, None));
            let (field, column) = feature_set.geometry_column(geometry_type, metadata)?;
            fields.push(field);
            columns.push(column);
        }

        let options = RecordBatchOptions::new().with_row_count(Some(feature_set.features.len()));
        Ok(RecordBatch::try_new_with_options(
            Arc::new(Schema::new(fields)),
            columns,
            &options,
        )?)
    }
}

#[cfg(test)]
mod tests;
