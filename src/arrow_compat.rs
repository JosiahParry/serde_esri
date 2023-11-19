use arrow::datatypes::{Field as AField, DataType, SchemaBuilder, Schema};

use crate::{
    features::Field,
    field_type::FieldType
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

// Takes a vector or Esri Fields and creates a schema
// of arrow field types
fn field_to_schema(fields: Vec<Field>) -> Schema {
    let mut sbuilder = SchemaBuilder::with_capacity(fields.len());

    for field in fields.into_iter() {
        let arrow_field = AField::from(field);
        sbuilder.push(arrow_field);
    }

    sbuilder.finish()

}