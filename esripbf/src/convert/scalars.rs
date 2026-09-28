//! Fields, values, spatial references, and envelopes, which convert one to one.

use crate::feature_collection_p_buffer::{self as pbf, value::ValueType};
use serde_esri::{
    features::{EsriValue, Field},
    field_type::FieldType,
    geometry::EsriEnvelope,
    spatial_reference::SpatialReference,
};

impl From<pbf::FieldType> for FieldType {
    fn from(value: pbf::FieldType) -> Self {
        match value {
            pbf::FieldType::EsriFieldTypeSmallInteger => FieldType::EsriFieldTypeSmallInteger,
            pbf::FieldType::EsriFieldTypeInteger => FieldType::EsriFieldTypeInteger,
            pbf::FieldType::EsriFieldTypeSingle => FieldType::EsriFieldTypeSingle,
            pbf::FieldType::EsriFieldTypeDouble => FieldType::EsriFieldTypeDouble,
            pbf::FieldType::EsriFieldTypeString => FieldType::EsriFieldTypeString,
            pbf::FieldType::EsriFieldTypeDate => FieldType::EsriFieldTypeDate,
            pbf::FieldType::EsriFieldTypeOid => FieldType::EsriFieldTypeOid,
            pbf::FieldType::EsriFieldTypeGeometry => FieldType::EsriFieldTypeGeometry,
            pbf::FieldType::EsriFieldTypeBlob => FieldType::EsriFieldTypeBlob,
            pbf::FieldType::EsriFieldTypeRaster => FieldType::EsriFieldTypeRaster,
            pbf::FieldType::EsriFieldTypeGuid => FieldType::EsriFieldTypeGuid,
            pbf::FieldType::EsriFieldTypeGlobalId => FieldType::EsriFieldTypeGlobalId,
            pbf::FieldType::EsriFieldTypeXml => FieldType::EsriFieldTypeXml,
            pbf::FieldType::EsriFieldTypeBigInteger => FieldType::EsriFieldTypeBigInteger,
            pbf::FieldType::EsriFieldTypeDateOnly => FieldType::EsriFieldTypeDateOnly,
            pbf::FieldType::EsriFieldTypeTimeOnly => FieldType::EsriFieldTypeTimeOnly,
            pbf::FieldType::EsriFieldTypeTimestampOffset => FieldType::EsriFieldTypeTimestampOffset,
        }
    }
}

impl From<pbf::Field> for Field {
    fn from(field: pbf::Field) -> Self {
        let text = |s: String| (!s.is_empty()).then_some(serde_json::Value::String(s));
        Field {
            field_type: field.field_type().into(),
            sqlType: Some(field.sql_type().as_str_name().to_string()),
            name: field.name,
            alias: (!field.alias.is_empty()).then_some(field.alias),
            domain: text(field.domain),
            defaultValue: text(field.default_value),
        }
    }
}

impl From<pbf::SpatialReference> for SpatialReference {
    fn from(sr: pbf::SpatialReference) -> Self {
        let wkid = |id: u32| i32::try_from(id).ok().filter(|id| *id != 0);
        let text = |s: String| (!s.is_empty()).then_some(s);
        SpatialReference {
            wkid: wkid(sr.wkid),
            latest_wkid: wkid(sr.lastest_wkid),
            vcs_wkid: wkid(sr.vcs_wkid),
            latest_vcs_wkid: wkid(sr.latest_vcs_wkid),
            wkt: text(sr.wkt),
            wkt2: text(sr.wkt2),
        }
    }
}

impl From<pbf::Value> for EsriValue {
    fn from(value: pbf::Value) -> Self {
        match value.value_type {
            None | Some(ValueType::NullValue(_)) => EsriValue::Null,
            Some(ValueType::StringValue(s)) => EsriValue::String(s),
            Some(ValueType::FloatValue(v)) => EsriValue::Float(f64::from(v)),
            Some(ValueType::DoubleValue(v)) => EsriValue::Float(v),
            Some(ValueType::SintValue(v)) => EsriValue::Int(i64::from(v)),
            Some(ValueType::UintValue(v)) => EsriValue::Int(i64::from(v)),
            Some(ValueType::Int64Value(v) | ValueType::Sint64Value(v)) => EsriValue::Int(v),
            Some(ValueType::Uint64Value(v)) => {
                i64::try_from(v).map_or(EsriValue::Float(v as f64), EsriValue::Int)
            }
            Some(ValueType::BoolValue(v)) => EsriValue::Bool(v),
        }
    }
}

impl From<pbf::Envelope> for EsriEnvelope {
    fn from(envelope: pbf::Envelope) -> Self {
        EsriEnvelope {
            xmin: envelope.x_min,
            ymin: envelope.y_min,
            xmax: envelope.x_max,
            ymax: envelope.y_max,
            spatialReference: envelope.spatial_reference.map(SpatialReference::from),
            ..Default::default()
        }
    }
}
