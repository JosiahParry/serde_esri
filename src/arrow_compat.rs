//! Compatibility with arrow-rs and geoarrow
//!
//! This module is enabled by the `geoarrow` feature. It provides a single function
//! `featureset_to_arrow()` which returns a `RecordBatch` containing arrays for each
//! field in the original `FeatureSet` struct and an additional field for geometry
//! if present.
//!
//! This feature implements the following geoarrow traits:
//!
//! - `EsriCoord<N>` implements `CoordTrait` and `PointTrait`
//! - `EsriPoint` implements `PointTrait`
//! - `EsriLineString<N>` implements `LineStringTrait`
//! - `EsriPolyline<N>` implements `MultiLineStringTrait`
//! - `EsriPolygon<N>` implements `PolygonTrait`
use crate::{
    features::{Feature, FeatureSet, Field},
    field_type::FieldType,
    geometry::EsriGeometry,
};

use std::{result::Result, sync::Arc};

use geoarrow::GeometryArrayTrait;
use serde_json::Value;
use std::collections::HashMap;

use arrow::{
    array::{
        make_builder, Array, ArrayBuilder, BooleanBuilder, Date32Builder, Date64Builder,
        Float32Builder, Float64Builder, Int16Builder, Int32Builder, Int64Builder, Int8Builder,
        NullBuilder, StringBuilder, UInt16Builder, UInt32Builder, UInt64Builder, UInt8Builder,
    },
    datatypes::{DataType, Field as AField, Schema, SchemaBuilder},
    record_batch::RecordBatch,
};

/// Given a `FeatureSet`, create a `RecordBatch`
pub fn featureset_to_arrow<const N: usize>(
    x: FeatureSet<N>,
) -> Result<RecordBatch, arrow::error::ArrowError> {
    let schema = field_to_schema(x.fields.unwrap());

    let (mut arrays, geometries) = create_array_vecs(&schema, x.features);

    let mut res_arrs = schema
        .fields()
        .iter()
        .map(|fi| {
            let arr = arrays.get_mut(fi.name()).unwrap();
            arr.1.finish()
        })
        .collect::<Vec<_>>();

    if x.geometryType.is_some() {
        // process geometries
        let (geo_field, geo_arr) = as_geoarrow_array(x.geometryType.unwrap().as_str(), geometries);

        // create a new schema builder
        let mut sb = SchemaBuilder::from(schema);

        // add the geometry field
        sb.push(geo_field);
        let schema = sb.finish();

        // extend res_arrs to include new geometry array
        res_arrs.push(geo_arr);

        RecordBatch::try_new(schema.into(), res_arrs)
    } else {
        RecordBatch::try_new(schema.into(), res_arrs)
    }
}

use geoarrow::table::GeoTable;

// TODO create my own error types
/// Given a `FeatureSet`, create a geoarrow `GeoTable`
pub fn featureset_to_geoarrow<const N: usize>(
    x: FeatureSet<N>,
) -> Result<GeoTable, geoarrow::error::GeoArrowError> {
    let arrow_res = featureset_to_arrow(x)?;
    let schema_ref = arrow_res.schema_ref().clone();
    let geometry_index = arrow_res.schema().fields.len();

    GeoTable::try_new(schema_ref, vec![arrow_res], geometry_index)
}

// convert an esri field to a new arrow field
impl From<Field> for AField {
    fn from(value: Field) -> Self {
        let dtype = match value.field_type {
            FieldType::EsriFieldTypeSmallInteger => DataType::Int16,
            FieldType::EsriFieldTypeInteger => DataType::Int32,
            FieldType::EsriFieldTypeSingle => DataType::Float32,
            FieldType::EsriFieldTypeDouble => DataType::Float64,
            FieldType::EsriFieldTypeString => DataType::Utf8,
            FieldType::EsriFieldTypeDate => DataType::Date64,
            FieldType::EsriFieldTypeOid => DataType::Int64,
            FieldType::EsriFieldTypeBlob => DataType::LargeBinary,
            FieldType::EsriFieldTypeGuid => DataType::Utf8,
            FieldType::EsriFieldTypeGlobalId => DataType::Utf8,
            FieldType::EsriFieldTypeXml => DataType::LargeUtf8,
            FieldType::EsriFieldTypeRaster => unimplemented!(),
            FieldType::EsriFieldTypeGeometry => unimplemented!(),
        };

        Self::new(value.name, dtype, true)
    }
}

// Takes a vector or Esri Fields and creates a Fields
// of arrow field types
fn field_to_schema(fields: Vec<Field>) -> Schema {
    let mut sbuilder = SchemaBuilder::with_capacity(fields.len());

    for field in fields.into_iter() {
        let arrow_field = AField::from(field);
        sbuilder.push(arrow_field);
    }
    sbuilder.finish()
}

// Takes a schema and a vector of features
// The features are processed into a tuple
// the first element is a hashmap containing the field name as keys
// and an array builder as the value
// the second element is a vectoor of geometry options
fn create_array_vecs<const N: usize>(
    //fields: &Fields,
    schema: &Schema,
    feats: Vec<Feature<N>>,
) -> (
    HashMap<&String, (&AField, Box<dyn ArrayBuilder>)>,
    Vec<Option<EsriGeometry<N>>>,
) {
    let n = feats.len();

    let mut map: HashMap<&String, (&AField, Box<dyn ArrayBuilder>)> = HashMap::new();

    let mut geometries = Vec::with_capacity(n);

    schema.fields.iter().for_each(|f| {
        let b = make_builder(f.data_type(), n);
        map.insert(f.name(), (&f, b));
    });

    feats.into_iter().for_each(|m| {
        let a1 = m.attributes.unwrap();

        a1.into_iter().for_each(|(k, v)| {
            let (field, builder) = map.get_mut(&k).unwrap();
            append_value(v, field, builder);
        });

        geometries.push(m.geometry);
    });

    (map, geometries)
}

fn as_geoarrow_array<const N: usize>(
    geom_type: &str,
    geoms: Vec<Option<EsriGeometry<N>>>,
) -> (Arc<AField>, Arc<dyn Array>) {
    match geom_type {
        "esriGeometryPoint" => {
            let res = geoms
                .into_iter()
                .map(|pi| match pi {
                    Some(pp) => pp.as_point(),
                    None => None,
                })
                .collect::<Vec<_>>();

            let arr = geoarrow::array::PointArray::from(res);
            (arr.extension_field(), arr.into_array_ref())
        }
        "esriGeometryMultipoint" => {
            let res = geoms
                .into_iter()
                .map(|pi| match pi {
                    Some(pp) => pp.as_multipoint(),
                    None => None,
                })
                .collect::<Vec<_>>();

            let arr = geoarrow::array::MultiPointArray::<i32>::from(res);
            (arr.extension_field(), arr.into_array_ref())
        }
        "esriGeometryPolyline" => {
            let res = geoms
                .into_iter()
                .map(|pi| match pi {
                    Some(pp) => pp.as_polyline(),
                    None => None,
                })
                .collect::<Vec<_>>();

            let arr = geoarrow::array::MultiLineStringArray::<i32>::from(res);
            (arr.extension_field(), arr.into_array_ref())
        }
        "esriGeometryPolygon" => {
            let res = geoms
                .into_iter()
                .map(|pi| match pi {
                    Some(pp) => pp.as_polygon(),
                    None => None,
                })
                .collect::<Vec<_>>();

            let arr = geoarrow::array::PolygonArray::<i32>::from(res);
            (arr.extension_field(), arr.into_array_ref())
        }
        _ => unimplemented!(),
    }
}

// take a field and a builder
// then match on the field to use downcast mut
fn append_value(v: Value, f: &AField, builder: &mut Box<dyn ArrayBuilder>) -> () {
    let bb = builder.as_any_mut();
    match f.data_type() {
        DataType::Null => {
            bb.downcast_mut::<NullBuilder>()
                .unwrap()
                .append_empty_value();
        }
        DataType::Boolean => {
            bb.downcast_mut::<BooleanBuilder>()
                .unwrap()
                .append_option(v.as_bool());
        }
        DataType::Int8 => {
            let builder = bb.downcast_mut::<Int8Builder>().unwrap();

            match v.as_i64() {
                Some(v) => builder.append_value(v as i8),
                None => builder.append_null(),
            };
        }
        DataType::Int16 => {
            let builder = bb.downcast_mut::<Int16Builder>().unwrap();

            match v.as_i64() {
                Some(v) => builder.append_value(v as i16),
                None => builder.append_null(),
            };
        }
        DataType::Int32 => {
            let builder = bb.downcast_mut::<Int32Builder>().unwrap();

            match v.as_i64() {
                Some(v) => builder.append_value(v as i32),
                None => builder.append_null(),
            };
        }
        DataType::Int64 => {
            bb.downcast_mut::<Int64Builder>()
                .unwrap()
                .append_option(v.as_i64());
        }
        DataType::UInt8 => {
            let builder = bb.downcast_mut::<UInt8Builder>().unwrap();

            match v.as_u64() {
                Some(v) => builder.append_value(v as u8),
                None => builder.append_null(),
            };
        }
        DataType::UInt16 => {
            let builder = bb.downcast_mut::<UInt16Builder>().unwrap();

            match v.as_u64() {
                Some(v) => builder.append_value(v as u16),
                None => builder.append_null(),
            };
        }
        DataType::UInt32 => {
            let builder = bb.downcast_mut::<UInt32Builder>().unwrap();

            match v.as_u64() {
                Some(v) => builder.append_value(v as u32),
                None => builder.append_null(),
            };
        }
        DataType::UInt64 => {
            bb.downcast_mut::<UInt64Builder>()
                .unwrap()
                .append_option(v.as_u64());
        }
        DataType::Float16 => {
            // bb.downcast_mut::<Float16Builder>()
            //     .unwrap()
            //     .append_value(v.as_f64().unwrap() as f16);
            // There is no 16 bit float in rust
            todo!()
        }
        DataType::Float32 => {
            let builder = bb.downcast_mut::<Float32Builder>().unwrap();

            match v.as_f64() {
                Some(v) => builder.append_value(v as f32),
                None => builder.append_null(),
            };
        }
        DataType::Float64 => {
            bb.downcast_mut::<Float64Builder>()
                .unwrap()
                .append_option(v.as_f64());
        }
        DataType::Timestamp(_, _) => todo!(),
        DataType::Date32 => {
            let builder = bb.downcast_mut::<Date32Builder>().unwrap();

            match v.as_i64() {
                Some(v) => builder.append_value((v / 100000_i64) as i32),
                None => builder.append_null(),
            };
        }
        DataType::Date64 => {
            let builder = bb.downcast_mut::<Date64Builder>().unwrap();

            match v.as_i64() {
                Some(v) => builder.append_value(v),
                None => builder.append_null(),
            };
        }
        DataType::Time32(_) => todo!(),
        DataType::Time64(_) => todo!(),
        DataType::Duration(_) => todo!(),
        DataType::Interval(_) => todo!(),
        DataType::Binary => todo!(),
        DataType::FixedSizeBinary(_) => todo!(),
        DataType::LargeBinary => todo!(),
        DataType::Utf8 => {
            bb.downcast_mut::<StringBuilder>()
                .unwrap()
                .append_option(v.as_str());
        }
        DataType::LargeUtf8 => {
            bb.downcast_mut::<StringBuilder>()
                .unwrap()
                .append_option(v.as_str());
        }
        DataType::List(_) => todo!(),
        DataType::FixedSizeList(_, _) => todo!(),
        DataType::LargeList(_) => todo!(),
        DataType::Struct(_) => todo!(),
        DataType::Union(_, _) => todo!(),
        DataType::Dictionary(_, _) => todo!(),
        DataType::Decimal128(_, _) => todo!(),
        DataType::Decimal256(_, _) => todo!(),
        DataType::Map(_, _) => todo!(),
        DataType::RunEndEncoded(_, _) => todo!(),
        _ => todo!(),
    }
}
