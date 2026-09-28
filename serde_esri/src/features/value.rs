//! Attribute values as the Feature specification defines them: strings, numbers, and booleans,
//! plus the nulls services return. Dates are numbers, milliseconds since the epoch in UTC.

use serde::{
    de::{self, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

/// One attribute value. Integers keep full 64-bit precision; arrays and objects, which the
/// specification does not allow, read as `Null`.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum EsriValue {
    #[default]
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl EsriValue {
    pub fn is_null(&self) -> bool {
        matches!(self, EsriValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            EsriValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// The value if it is an integer.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            EsriValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// The value if it is a number, integers widened to floats.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            EsriValue::Int(i) => Some(*i as f64),
            EsriValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            EsriValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl From<bool> for EsriValue {
    fn from(v: bool) -> Self {
        EsriValue::Bool(v)
    }
}

impl From<i64> for EsriValue {
    fn from(v: i64) -> Self {
        EsriValue::Int(v)
    }
}

impl From<i32> for EsriValue {
    fn from(v: i32) -> Self {
        EsriValue::Int(i64::from(v))
    }
}

impl From<f64> for EsriValue {
    fn from(v: f64) -> Self {
        EsriValue::Float(v)
    }
}

impl From<String> for EsriValue {
    fn from(v: String) -> Self {
        EsriValue::String(v)
    }
}

impl From<&str> for EsriValue {
    fn from(v: &str) -> Self {
        EsriValue::String(v.to_owned())
    }
}

impl<T: Into<EsriValue>> From<Option<T>> for EsriValue {
    fn from(v: Option<T>) -> Self {
        v.map_or(EsriValue::Null, Into::into)
    }
}

impl Serialize for EsriValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            EsriValue::Null => serializer.serialize_unit(),
            EsriValue::Bool(b) => serializer.serialize_bool(*b),
            EsriValue::Int(i) => serializer.serialize_i64(*i),
            EsriValue::Float(f) => serializer.serialize_f64(*f),
            EsriValue::String(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for EsriValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = EsriValue;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string, number, boolean, or null")
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<EsriValue, E> {
                Ok(EsriValue::Bool(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<EsriValue, E> {
                Ok(EsriValue::Int(v))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<EsriValue, E> {
                Ok(i64::try_from(v).map_or(EsriValue::Float(v as f64), EsriValue::Int))
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<EsriValue, E> {
                Ok(EsriValue::Float(v))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<EsriValue, E> {
                Ok(EsriValue::String(v.to_owned()))
            }

            fn visit_string<E: de::Error>(self, v: String) -> Result<EsriValue, E> {
                Ok(EsriValue::String(v))
            }

            fn visit_unit<E: de::Error>(self) -> Result<EsriValue, E> {
                Ok(EsriValue::Null)
            }

            fn visit_none<E: de::Error>(self) -> Result<EsriValue, E> {
                Ok(EsriValue::Null)
            }

            fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<EsriValue, D::Error> {
                EsriValue::deserialize(d)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<EsriValue, A::Error> {
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(EsriValue::Null)
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<EsriValue, A::Error> {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(EsriValue::Null)
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests;
