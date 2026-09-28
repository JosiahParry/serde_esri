//! Attribute columns built straight from feature JSON, typed by the FeatureSet's fields.
//!
//! Values that do not fit their column, such as text in an integer field, become nulls.

use arrow_array::{
    builder::{
        ArrayBuilder, Float32Builder, Float64Builder, Int16Builder, Int32Builder, Int64Builder,
        LargeStringBuilder, StringBuilder, TimestampMillisecondBuilder,
    },
    ArrayRef,
};
use serde::{
    de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserialize,
};
use std::{borrow::Cow, collections::HashMap, fmt, sync::Arc};

/// A field as a FeatureSet declares it.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct RawField {
    pub(super) name: String,
    #[serde(rename = "type")]
    pub(super) field_type: String,
}

/// An Arrow builder for one attribute column.
pub(super) enum ColumnBuilder {
    Int16(Int16Builder),
    Int32(Int32Builder),
    Int64(Int64Builder),
    Float32(Float32Builder),
    Float64(Float64Builder),
    Utf8(StringBuilder),
    LargeUtf8(LargeStringBuilder),
    Timestamp(TimestampMillisecondBuilder),
}

impl ColumnBuilder {
    /// The builder for an Esri field type, or `None` for geometry, blob, and raster fields.
    /// Unrecognized types, including date-only, time-only, and offset timestamps, are text.
    pub(super) fn new(field_type: &str, capacity: usize) -> Option<Self> {
        Some(match field_type {
            "esriFieldTypeSmallInteger" => ColumnBuilder::Int16(Int16Builder::with_capacity(capacity)),
            "esriFieldTypeInteger" => ColumnBuilder::Int32(Int32Builder::with_capacity(capacity)),
            "esriFieldTypeOID" | "esriFieldTypeBigInteger" => {
                ColumnBuilder::Int64(Int64Builder::with_capacity(capacity))
            }
            "esriFieldTypeSingle" => ColumnBuilder::Float32(Float32Builder::with_capacity(capacity)),
            "esriFieldTypeDouble" => ColumnBuilder::Float64(Float64Builder::with_capacity(capacity)),
            "esriFieldTypeDate" => {
                ColumnBuilder::Timestamp(TimestampMillisecondBuilder::with_capacity(capacity))
            }
            "esriFieldTypeXML" => {
                ColumnBuilder::LargeUtf8(LargeStringBuilder::with_capacity(capacity, capacity * 16))
            }
            "esriFieldTypeGeometry" | "esriFieldTypeBlob" | "esriFieldTypeRaster" => return None,
            _ => ColumnBuilder::Utf8(StringBuilder::with_capacity(capacity, capacity * 16)),
        })
    }

    pub(super) fn len(&self) -> usize {
        match self {
            ColumnBuilder::Int16(b) => b.len(),
            ColumnBuilder::Int32(b) => b.len(),
            ColumnBuilder::Int64(b) => b.len(),
            ColumnBuilder::Float32(b) => b.len(),
            ColumnBuilder::Float64(b) => b.len(),
            ColumnBuilder::Utf8(b) => b.len(),
            ColumnBuilder::LargeUtf8(b) => b.len(),
            ColumnBuilder::Timestamp(b) => b.len(),
        }
    }

    pub(super) fn append_null(&mut self) {
        match self {
            ColumnBuilder::Int16(b) => b.append_null(),
            ColumnBuilder::Int32(b) => b.append_null(),
            ColumnBuilder::Int64(b) => b.append_null(),
            ColumnBuilder::Float32(b) => b.append_null(),
            ColumnBuilder::Float64(b) => b.append_null(),
            ColumnBuilder::Utf8(b) => b.append_null(),
            ColumnBuilder::LargeUtf8(b) => b.append_null(),
            ColumnBuilder::Timestamp(b) => b.append_null(),
        }
    }

    fn append_i64(&mut self, v: i64) {
        match self {
            ColumnBuilder::Int16(b) => b.append_option(i16::try_from(v).ok()),
            ColumnBuilder::Int32(b) => b.append_option(i32::try_from(v).ok()),
            ColumnBuilder::Int64(b) => b.append_value(v),
            ColumnBuilder::Timestamp(b) => b.append_value(v),
            ColumnBuilder::Float32(b) => b.append_value(v as f32),
            ColumnBuilder::Float64(b) => b.append_value(v as f64),
            ColumnBuilder::Utf8(b) => b.append_value(v.to_string()),
            ColumnBuilder::LargeUtf8(b) => b.append_value(v.to_string()),
        }
    }

    /// Whole floats go into integer columns when they fit exactly; others become null.
    fn append_f64(&mut self, v: f64) {
        let whole = (v.fract() == 0.0).then_some(v);
        let int64 = whole.map(|w| w as i64).filter(|&i| i as f64 == v);
        match self {
            ColumnBuilder::Int16(b) => b.append_option(int64.and_then(|i| i16::try_from(i).ok())),
            ColumnBuilder::Int32(b) => b.append_option(int64.and_then(|i| i32::try_from(i).ok())),
            ColumnBuilder::Int64(b) => b.append_option(int64),
            ColumnBuilder::Timestamp(b) => b.append_option(int64),
            ColumnBuilder::Float32(b) => b.append_value(v as f32),
            ColumnBuilder::Float64(b) => b.append_value(v),
            ColumnBuilder::Utf8(b) => b.append_value(v.to_string()),
            ColumnBuilder::LargeUtf8(b) => b.append_value(v.to_string()),
        }
    }

    fn append_str(&mut self, v: &str) {
        match self {
            ColumnBuilder::Int16(b) => b.append_option(v.parse().ok()),
            ColumnBuilder::Int32(b) => b.append_option(v.parse().ok()),
            ColumnBuilder::Int64(b) => b.append_option(v.parse().ok()),
            ColumnBuilder::Timestamp(b) => b.append_option(v.parse().ok()),
            ColumnBuilder::Float32(b) => b.append_option(v.parse().ok()),
            ColumnBuilder::Float64(b) => b.append_option(v.parse().ok()),
            ColumnBuilder::Utf8(b) => b.append_value(v),
            ColumnBuilder::LargeUtf8(b) => b.append_value(v),
        }
    }

    fn append_bool(&mut self, v: bool) {
        match self {
            ColumnBuilder::Utf8(b) => b.append_value(if v { "true" } else { "false" }),
            ColumnBuilder::LargeUtf8(b) => b.append_value(if v { "true" } else { "false" }),
            ColumnBuilder::Timestamp(b) => b.append_null(),
            numeric => numeric.append_i64(i64::from(v)),
        }
    }

    /// The finished column; dates are UTC millisecond timestamps.
    pub(super) fn finish(&mut self) -> ArrayRef {
        match self {
            ColumnBuilder::Int16(b) => Arc::new(b.finish()),
            ColumnBuilder::Int32(b) => Arc::new(b.finish()),
            ColumnBuilder::Int64(b) => Arc::new(b.finish()),
            ColumnBuilder::Float32(b) => Arc::new(b.finish()),
            ColumnBuilder::Float64(b) => Arc::new(b.finish()),
            ColumnBuilder::Utf8(b) => Arc::new(b.finish()),
            ColumnBuilder::LargeUtf8(b) => Arc::new(b.finish()),
            ColumnBuilder::Timestamp(b) => Arc::new(b.finish().with_timezone("UTC")),
        }
    }
}

/// Appends one JSON attribute value to a column.
struct AppendValue<'b>(&'b mut ColumnBuilder);

impl<'de> DeserializeSeed<'de> for AppendValue<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for AppendValue<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an attribute value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<(), E> {
        self.0.append_bool(v);
        Ok(())
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<(), E> {
        self.0.append_i64(v);
        Ok(())
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<(), E> {
        match i64::try_from(v) {
            Ok(v) => self.0.append_i64(v),
            Err(_) => self.0.append_f64(v as f64),
        }
        Ok(())
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<(), E> {
        self.0.append_f64(v);
        Ok(())
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<(), E> {
        self.0.append_str(v);
        Ok(())
    }

    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        self.0.append_null();
        Ok(())
    }

    fn visit_none<E: de::Error>(self) -> Result<(), E> {
        self.0.append_null();
        Ok(())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        self.0.append_null();
        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
        self.0.append_null();
        Ok(())
    }
}

/// A key borrowed from the input when it has no escapes.
struct Key<'de>(Cow<'de, str>);

impl<'de> Deserialize<'de> for Key<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = Key<'de>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a field name")
            }

            fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Key<'de>, E> {
                Ok(Key(Cow::Borrowed(v)))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Key<'de>, E> {
                Ok(Key(Cow::Owned(v.to_owned())))
            }
        }

        deserializer.deserialize_str(KeyVisitor)
    }
}

/// Appends a feature's `attributes` to their columns, looked up by field name.
/// Unknown names and repeated keys are skipped; the caller fills columns left unset.
pub(super) struct AttributesSeed<'b> {
    pub(super) columns: &'b mut [ColumnBuilder],
    pub(super) lookup: &'b HashMap<String, usize>,
    pub(super) row: usize,
}

impl<'de> DeserializeSeed<'de> for AttributesSeed<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for AttributesSeed<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an attributes object")
    }

    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        while let Some(Key(name)) = map.next_key()? {
            let column = self
                .lookup
                .get(name.as_ref())
                .and_then(|&i| self.columns.get_mut(i))
                .filter(|column| column.len() == self.row);
            match column {
                Some(column) => map.next_value_seed(AppendValue(column))?,
                None => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}
