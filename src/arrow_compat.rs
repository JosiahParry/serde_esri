use std::collections::HashMap;

use arrow::array::Array;
use arrow::{
    array::{
        make_builder, ArrayBuilder, BooleanBuilder, Float32Builder, Float64Builder,
        Int16Builder, Int32Builder, Int64Builder, Int8Builder, NullBuilder, StringBuilder,
        UInt16Builder, UInt32Builder, UInt64Builder, UInt8Builder,
    },
    datatypes::{
        DataType,
        Field as AField,
        // Schema,
        Fields,
        SchemaBuilder,
    }
};
use serde_json::Value;

// use arrow::array::builder::StructBuilder;

use crate::features::FeatureSet;
use crate::{
    features::{Feature, Field},
    field_type::FieldType,
};

// convert a field to a new field
impl From<Field> for AField {
    fn from(value: Field) -> Self {
        let dtype = match value.field_type {
            FieldType::EsriFieldTypeSmallInteger => DataType::Int16,
            FieldType::EsriFieldTypeInteger => DataType::Int32,
            FieldType::EsriFieldTypeSingle => DataType::Float32,
            FieldType::EsriFieldTypeDouble => DataType::Float64,
            FieldType::EsriFieldTypeString => DataType::Utf8,
            FieldType::EsriFieldTypeDate => DataType::Date32,
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
fn field_to_schema(fields: Vec<Field>) -> Fields {
    let mut sbuilder = SchemaBuilder::with_capacity(fields.len());

    for field in fields.into_iter() {
        let arrow_field = AField::from(field);
        sbuilder.push(arrow_field);
    }
    sbuilder.finish().fields
}

fn create_array_vecs<const N: usize>(
    fields: &Fields,
    feats: Vec<Feature<N>>,
) -> Vec<std::sync::Arc<dyn Array>> {
    let n = feats.len();
    let mut map: HashMap<&String, (&AField, Box<dyn ArrayBuilder>)> = HashMap::new();

    fields.iter().for_each(|f| {
        let b = make_builder(f.data_type(), n);
        map.insert(f.name(), (&f, b));
    });

    // let a1 = feats[0].attributes.clone().unwrap();

    feats.into_iter().for_each(|m| {
        let a1 = m.attributes.unwrap();

        a1.into_iter().for_each(|(k, v)| {
            let (field, builder) = map.get_mut(&k).unwrap();
            append_value(v, field, builder);
        });
    });

    let res = map
        .into_iter()
        .map(|mut bi| {
            let arr = bi.1 .1.finish();
            arr
        })
        .collect::<Vec<_>>();

    res
}

pub fn featureset_to_arrow<const N: usize>(x: FeatureSet<N>) -> Vec<std::sync::Arc<dyn Array>> {
    let fields = field_to_schema(x.fields.unwrap());
    create_array_vecs(&fields, x.features)
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
                .append_value(v.as_bool().unwrap());
        }
        DataType::Int8 => {
            bb.downcast_mut::<Int8Builder>()
                .unwrap()
                .append_value(v.as_i64().unwrap() as i8);
        }
        DataType::Int16 => {
            bb.downcast_mut::<Int16Builder>()
                .unwrap()
                .append_value(v.as_i64().unwrap() as i16);
        }
        DataType::Int32 => {
            bb.downcast_mut::<Int32Builder>()
                .unwrap()
                .append_value(v.as_i64().unwrap() as i32);
        }
        DataType::Int64 => {
            bb.downcast_mut::<Int64Builder>()
                .unwrap()
                .append_value(v.as_i64().unwrap());
        }
        DataType::UInt8 => {
            bb.downcast_mut::<UInt8Builder>()
                .unwrap()
                .append_value(v.as_u64().unwrap() as u8);
        }
        DataType::UInt16 => {
            bb.downcast_mut::<UInt16Builder>()
                .unwrap()
                .append_value(v.as_u64().unwrap() as u16);
        }
        DataType::UInt32 => {
            bb.downcast_mut::<UInt32Builder>()
                .unwrap()
                .append_value(v.as_u64().unwrap() as u32);
        }
        DataType::UInt64 => {
            bb.downcast_mut::<UInt64Builder>()
                .unwrap()
                .append_value(v.as_u64().unwrap());
        }
        DataType::Float16 => {
            // bb.downcast_mut::<Float16Builder>()
            //     .unwrap()
            //     .append_value(v.as_f64().unwrap() as f16);
            // There is no 16 bit float in rust
            todo!()
        }
        DataType::Float32 => {
            bb.downcast_mut::<Float32Builder>()
                .unwrap()
                .append_value(v.as_f64().unwrap() as f32);
        }
        DataType::Float64 => {
            bb.downcast_mut::<Float64Builder>()
                .unwrap()
                .append_value(v.as_f64().unwrap());
        }
        DataType::Timestamp(_, _) => todo!(),
        DataType::Date32 => todo!(),
        DataType::Date64 => todo!(),
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
                .append_value(v.as_str().unwrap());
        }
        DataType::LargeUtf8 => {
            bb.downcast_mut::<StringBuilder>()
                .unwrap()
                .append_value(v.as_str().unwrap());
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
    }
}
