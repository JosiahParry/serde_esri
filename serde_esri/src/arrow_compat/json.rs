//! Builds a [`RecordBatch`] straight from FeatureSet JSON, without materializing features.
//!
//! The FeatureSet's `geometryType`, `hasZ`, `hasM`, and `fields` decide the columns. Features
//! stream into them as they are read, which takes one pass when those keys precede `features`,
//! as services write them. When a key that changes the columns follows `features`, the input is
//! parsed again with every key known.

use crate::{
    arrow_compat::{
        columns::{AttributesSeed, ColumnBuilder, RawField},
        geometry::{GeometryBuilder, GeometrySeed, GeometryType},
        ToArrowError,
    },
    spatial_reference::SpatialReference,
};
use arrow_array::{RecordBatch, RecordBatchOptions};
use arrow_schema::{Field, Schema};
use geoarrow_schema::{Crs, Dimension, Metadata};
use serde::{
    de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserialize,
};
use std::{collections::HashMap, fmt, sync::Arc};

/// FeatureSet JSON bytes, as a feature service returns them.
#[derive(Clone, Copy, Debug)]
pub struct FeatureSetJson<'a>(pub &'a [u8]);

/// The keys that decide a FeatureSet's columns.
#[derive(Clone, Debug, Default, PartialEq)]
struct Header {
    geometry_type: Option<String>,
    has_z: Option<bool>,
    has_m: Option<bool>,
    fields: Vec<RawField>,
}

/// The error object a service returns in place of a FeatureSet.
#[derive(Deserialize)]
struct ServiceError {
    code: Option<i64>,
    message: Option<String>,
}

/// Builders for every column of a FeatureSet.
struct Builders {
    names: Vec<String>,
    columns: Vec<ColumnBuilder>,
    lookup: HashMap<String, usize>,
    geometry: Option<GeometryBuilder>,
    rows: usize,
}

impl Builders {
    fn new(header: &Header) -> Result<Self, ToArrowError> {
        let mut builders = Builders {
            names: Vec::new(),
            columns: Vec::new(),
            lookup: HashMap::new(),
            geometry: None,
            rows: 0,
        };
        for field in &header.fields {
            let Some(column) = ColumnBuilder::from_name(&field.field_type, 0) else {
                continue;
            };
            if builders.lookup.contains_key(&field.name) {
                continue;
            }
            builders.lookup.insert(field.name.clone(), builders.columns.len());
            builders.names.push(field.name.clone());
            builders.columns.push(column);
        }
        if let Some(geometry_type) = &header.geometry_type {
            let dim = match (header.has_z == Some(true), header.has_m == Some(true)) {
                (false, false) => Dimension::XY,
                (true, false) => Dimension::XYZ,
                (false, true) => Dimension::XYM,
                (true, true) => Dimension::XYZM,
            };
            let geometry_type = GeometryType::try_from(geometry_type.as_str())?;
            builders.geometry = Some(GeometryBuilder::new(geometry_type, dim, 0));
        }
        Ok(builders)
    }

    /// Ends the current row, filling nulls into columns the feature left unset.
    fn finish_row(&mut self) {
        let row = self.rows;
        for column in self.columns.iter_mut().filter(|c| c.len() == row) {
            column.append_null();
        }
        if let Some(geometry) = self.geometry.as_mut().filter(|g| g.len() == row) {
            geometry.append_null();
        }
        self.rows += 1;
    }
}

/// The keys of a FeatureSet object this reader uses.
enum TopKey {
    GeometryType,
    HasZ,
    HasM,
    SpatialReference,
    Fields,
    Features,
    Error,
    Other,
}

impl<'de> Deserialize<'de> for TopKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor;

        impl Visitor<'_> for KeyVisitor {
            type Value = TopKey;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a FeatureSet key")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<TopKey, E> {
                Ok(match v {
                    "geometryType" => TopKey::GeometryType,
                    "hasZ" => TopKey::HasZ,
                    "hasM" => TopKey::HasM,
                    "spatialReference" => TopKey::SpatialReference,
                    "fields" => TopKey::Fields,
                    "features" => TopKey::Features,
                    "error" => TopKey::Error,
                    _ => TopKey::Other,
                })
            }
        }

        deserializer.deserialize_str(KeyVisitor)
    }
}

/// The keys of a feature object this reader uses.
enum FeatureKey {
    Attributes,
    Geometry,
    Other,
}

impl<'de> Deserialize<'de> for FeatureKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor;

        impl Visitor<'_> for KeyVisitor {
            type Value = FeatureKey;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a feature key")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<FeatureKey, E> {
                Ok(match v {
                    "attributes" => FeatureKey::Attributes,
                    "geometry" => FeatureKey::Geometry,
                    _ => FeatureKey::Other,
                })
            }
        }

        deserializer.deserialize_str(KeyVisitor)
    }
}

/// Appends one feature to the builders.
struct FeatureSeed<'b>(&'b mut Builders);

impl<'de> DeserializeSeed<'de> for FeatureSeed<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for FeatureSeed<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a feature object")
    }

    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        let b = self.0;
        let row = b.rows;
        while let Some(key) = map.next_key::<FeatureKey>()? {
            match key {
                FeatureKey::Attributes => map.next_value_seed(AttributesSeed {
                    columns: &mut b.columns,
                    lookup: &b.lookup,
                    row,
                })?,
                FeatureKey::Geometry => match b.geometry.as_mut().filter(|g| g.len() == row) {
                    Some(geometry) => map.next_value_seed(GeometrySeed(geometry))?,
                    None => {
                        map.next_value::<IgnoredAny>()?;
                    }
                },
                FeatureKey::Other => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

/// Appends every feature of the `features` array.
struct FeaturesSeed<'b>(&'b mut Builders);

impl<'de> DeserializeSeed<'de> for FeaturesSeed<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for FeaturesSeed<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an array of features")
    }

    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while seq.next_element_seed(FeatureSeed(&mut *self.0))?.is_some() {
            self.0.finish_row();
        }
        Ok(())
    }
}

/// One read of the input. With a `known` header its keys are skipped and features use it;
/// otherwise features use the keys read before them.
struct Pass<'h> {
    known: Option<&'h Header>,
    header: Header,
    spatial_reference: Option<SpatialReference>,
    service_error: Option<ServiceError>,
    /// The builders and the header they were made from.
    built: Option<(Header, Builders)>,
    /// A failure outside serde's error type, which stops the read.
    failure: Option<ToArrowError>,
}

impl<'de> Visitor<'de> for &mut Pass<'_> {
    type Value = ();

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a FeatureSet object")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        let reads_header = self.known.is_none();
        while let Some(key) = map.next_key::<TopKey>()? {
            match key {
                TopKey::GeometryType if reads_header => self.header.geometry_type = map.next_value()?,
                TopKey::HasZ if reads_header => self.header.has_z = map.next_value()?,
                TopKey::HasM if reads_header => self.header.has_m = map.next_value()?,
                TopKey::Fields if reads_header => {
                    self.header.fields = map.next_value::<Option<_>>()?.unwrap_or_default();
                }
                TopKey::SpatialReference => self.spatial_reference = map.next_value()?,
                TopKey::Error => self.service_error = map.next_value()?,
                TopKey::Features => {
                    let header = self.known.cloned().unwrap_or_else(|| self.header.clone());
                    let mut builders = match Builders::new(&header) {
                        Ok(builders) => builders,
                        Err(e) => {
                            self.failure = Some(e);
                            return Err(de::Error::custom("unsupported FeatureSet"));
                        }
                    };
                    map.next_value_seed(FeaturesSeed(&mut builders))?;
                    self.built = Some((header, builders));
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

impl<'h> Pass<'h> {
    fn run(bytes: &[u8], known: Option<&'h Header>) -> Result<Self, ToArrowError> {
        let mut pass = Pass {
            known,
            header: Header::default(),
            spatial_reference: None,
            service_error: None,
            built: None,
            failure: None,
        };
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        let read = (&mut deserializer)
            .deserialize_map(&mut pass)
            .and_then(|()| deserializer.end());
        if let Some(failure) = pass.failure.take() {
            return Err(failure);
        }
        read?;
        if let Some(error) = pass.service_error.take() {
            return Err(ToArrowError::Service {
                code: error.code,
                message: error.message.unwrap_or_default(),
            });
        }
        Ok(pass)
    }
}

impl TryFrom<FeatureSetJson<'_>> for RecordBatch {
    type Error = ToArrowError;

    fn try_from(json: FeatureSetJson<'_>) -> Result<Self, Self::Error> {
        let first = Pass::run(json.0, None)?;
        let header = first.header;
        let spatial_reference = first.spatial_reference;
        let mut builders = match first.built {
            Some((used, builders)) if used == header => builders,
            Some(_) => Pass::run(json.0, Some(&header))?
                .built
                .map(|(_, builders)| builders)
                .ok_or(ToArrowError::GeometryTypeMismatch)?,
            None => Builders::new(&header)?,
        };

        let mut fields = Vec::with_capacity(builders.columns.len() + 1);
        let mut arrays = Vec::with_capacity(builders.columns.len() + 1);
        for (name, column) in builders.names.iter().zip(builders.columns.iter_mut()) {
            let array = column.finish();
            fields.push(Field::new(name, array.data_type().clone(), true));
            arrays.push(array);
        }
        if let Some(geometry) = builders.geometry {
            let crs = spatial_reference.as_ref().map(Crs::from).unwrap_or_default();
            let (field, array) = geometry.finish(Arc::new(Metadata::new(crs, None)))?;
            fields.push(field);
            arrays.push(array);
        }

        let options = RecordBatchOptions::new().with_row_count(Some(builders.rows));
        Ok(RecordBatch::try_new_with_options(
            Arc::new(Schema::new(fields)),
            arrays,
            &options,
        )?)
    }
}

#[cfg(test)]
mod tests;
