use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FieldType {
    EsriFieldTypeSmallInteger = 0,
    EsriFieldTypeInteger = 1,
    EsriFieldTypeSingle = 2,
    EsriFieldTypeDouble = 3,
    EsriFieldTypeString = 4,
    EsriFieldTypeDate = 5,
    EsriFieldTypeOid = 6,
    EsriFieldTypeGeometry = 7,
    EsriFieldTypeBlob = 8,
    EsriFieldTypeRaster = 9,
    EsriFieldTypeGuid = 10,
    EsriFieldTypeGlobalId = 11,
    EsriFieldTypeXml = 12,
}

impl FieldType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            FieldType::EsriFieldTypeSmallInteger => "esriFieldTypeSmallInteger",
            FieldType::EsriFieldTypeInteger => "esriFieldTypeInteger",
            FieldType::EsriFieldTypeSingle => "esriFieldTypeSingle",
            FieldType::EsriFieldTypeDouble => "esriFieldTypeDouble",
            FieldType::EsriFieldTypeString => "esriFieldTypeString",
            FieldType::EsriFieldTypeDate => "esriFieldTypeDate",
            FieldType::EsriFieldTypeOid => "esriFieldTypeOID",
            FieldType::EsriFieldTypeGeometry => "esriFieldTypeGeometry",
            FieldType::EsriFieldTypeBlob => "esriFieldTypeBlob",
            FieldType::EsriFieldTypeRaster => "esriFieldTypeRaster",
            FieldType::EsriFieldTypeGuid => "esriFieldTypeGUID",
            FieldType::EsriFieldTypeGlobalId => "esriFieldTypeGlobalID",
            FieldType::EsriFieldTypeXml => "esriFieldTypeXML",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "esriFieldTypeSmallInteger" => Some(Self::EsriFieldTypeSmallInteger),
            "esriFieldTypeInteger" => Some(Self::EsriFieldTypeInteger),
            "esriFieldTypeSingle" => Some(Self::EsriFieldTypeSingle),
            "esriFieldTypeDouble" => Some(Self::EsriFieldTypeDouble),
            "esriFieldTypeString" => Some(Self::EsriFieldTypeString),
            "esriFieldTypeDate" => Some(Self::EsriFieldTypeDate),
            "esriFieldTypeOID" => Some(Self::EsriFieldTypeOid),
            "esriFieldTypeGeometry" => Some(Self::EsriFieldTypeGeometry),
            "esriFieldTypeBlob" => Some(Self::EsriFieldTypeBlob),
            "esriFieldTypeRaster" => Some(Self::EsriFieldTypeRaster),
            "esriFieldTypeGUID" => Some(Self::EsriFieldTypeGuid),
            "esriFieldTypeGlobalID" => Some(Self::EsriFieldTypeGlobalId),
            "esriFieldTypeXML" => Some(Self::EsriFieldTypeXml),
            _ => None,
        }
    }
}


impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str_name())
    }
}
impl std::str::FromStr for FieldType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match FieldType::from_str_name(s) {
            Some(f) => Ok(f),
            None => Err(String::from("Cannot determine `FieldType`")),
        }
    }
}