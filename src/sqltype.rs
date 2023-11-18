// taken from
// https://github.com/R-ArcGIS/arcpbf/blob/main/src/rust/esripbf/src/esri_p_buffer.rs#L31
/// FieldType
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd)]
pub enum SqlType {
    BigInt = 0,
    Binary = 1,
    Bit = 2,
    Char = 3,
    Date = 4,
    Decimal = 5,
    Double = 6,
    Float = 7,
    Geometry = 8,
    Guid = 9,
    Integer = 10,
    LongNVarchar = 11,
    LongVarbinary = 12,
    LongVarchar = 13,
    NChar = 14,
    NVarchar = 15,
    Other = 16,
    Real = 17,
    SmallInt = 18,
    SqlXml = 19,
    Time = 20,
    Timestamp = 21,
    Timestamp2 = 22,
    TinyInt = 23,
    Varbinary = 24,
    Varchar = 25,
}

impl SqlType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SqlType::BigInt => "sqlTypeBigInt",
            SqlType::Binary => "sqlTypeBinary",
            SqlType::Bit => "sqlTypeBit",
            SqlType::Char => "sqlTypeChar",
            SqlType::Date => "sqlTypeDate",
            SqlType::Decimal => "sqlTypeDecimal",
            SqlType::Double => "sqlTypeDouble",
            SqlType::Float => "sqlTypeFloat",
            SqlType::Geometry => "sqlTypeGeometry",
            SqlType::Guid => "sqlTypeGUID",
            SqlType::Integer => "sqlTypeInteger",
            SqlType::LongNVarchar => "sqlTypeLongNVarchar",
            SqlType::LongVarbinary => "sqlTypeLongVarbinary",
            SqlType::LongVarchar => "sqlTypeLongVarchar",
            SqlType::NChar => "sqlTypeNChar",
            SqlType::NVarchar => "sqlTypeNVarchar",
            SqlType::Other => "sqlTypeOther",
            SqlType::Real => "sqlTypeReal",
            SqlType::SmallInt => "sqlTypeSmallInt",
            SqlType::SqlXml => "sqlTypeSqlXml",
            SqlType::Time => "sqlTypeTime",
            SqlType::Timestamp => "sqlTypeTimestamp",
            SqlType::Timestamp2 => "sqlTypeTimestamp2",
            SqlType::TinyInt => "sqlTypeTinyInt",
            SqlType::Varbinary => "sqlTypeVarbinary",
            SqlType::Varchar => "sqlTypeVarchar",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "sqlTypeBigInt" => Some(Self::BigInt),
            "sqlTypeBinary" => Some(Self::Binary),
            "sqlTypeBit" => Some(Self::Bit),
            "sqlTypeChar" => Some(Self::Char),
            "sqlTypeDate" => Some(Self::Date),
            "sqlTypeDecimal" => Some(Self::Decimal),
            "sqlTypeDouble" => Some(Self::Double),
            "sqlTypeFloat" => Some(Self::Float),
            "sqlTypeGeometry" => Some(Self::Geometry),
            "sqlTypeGUID" => Some(Self::Guid),
            "sqlTypeInteger" => Some(Self::Integer),
            "sqlTypeLongNVarchar" => Some(Self::LongNVarchar),
            "sqlTypeLongVarbinary" => Some(Self::LongVarbinary),
            "sqlTypeLongVarchar" => Some(Self::LongVarchar),
            "sqlTypeNChar" => Some(Self::NChar),
            "sqlTypeNVarchar" => Some(Self::NVarchar),
            "sqlTypeOther" => Some(Self::Other),
            "sqlTypeReal" => Some(Self::Real),
            "sqlTypeSmallInt" => Some(Self::SmallInt),
            "sqlTypeSqlXml" => Some(Self::SqlXml),
            "sqlTypeTime" => Some(Self::Time),
            "sqlTypeTimestamp" => Some(Self::Timestamp),
            "sqlTypeTimestamp2" => Some(Self::Timestamp2),
            "sqlTypeTinyInt" => Some(Self::TinyInt),
            "sqlTypeVarbinary" => Some(Self::Varbinary),
            "sqlTypeVarchar" => Some(Self::Varchar),
            _ => None,
        }
    }
}
