use crate::error::Error;
use arrow_array::{cast::AsArray, ArrayRef, RecordBatch};
use esripbf::esri_p_buffer::feature_collection_p_buffer as pbf;
use parquet::{file::metadata::FileMetaData, schema::types::SchemaDescriptor};
use serde::Deserialize;
use serde_esri::{
    enginex::description::{Attribute, VertexDescription},
    geometry::EsriEnvelope,
};
use serde_json::value::RawValue;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ColumnPath {
    Field(String),
    Child(String, String),
}

impl ColumnPath {
    fn parts(&self) -> Vec<&str> {
        match self {
            ColumnPath::Field(name) => vec![name],
            ColumnPath::Child(group, name) => vec![group, name],
        }
    }

    pub fn leaf(&self, schema: &SchemaDescriptor) -> Result<usize, Error> {
        schema
            .columns()
            .iter()
            .position(|c| c.path().parts().iter().map(String::as_str).eq(self.parts()))
            .ok_or_else(|| Error::MissingColumn(self.to_string()))
    }

    pub fn column<'a>(&self, batch: &'a RecordBatch) -> Result<&'a ArrayRef, Error> {
        let missing = || Error::MissingColumn(self.to_string());
        match self {
            ColumnPath::Field(name) => batch.column_by_name(name).ok_or_else(missing),
            ColumnPath::Child(group, name) => batch
                .column_by_name(group)
                .and_then(|group| group.as_struct_opt())
                .and_then(|group| group.column_by_name(name))
                .ok_or_else(missing),
        }
    }

    fn nest(&mut self, group: &str) {
        if let ColumnPath::Field(name) = self {
            *self = ColumnPath::Child(group.to_string(), std::mem::take(name));
        }
    }
}

impl std::fmt::Display for ColumnPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parts().join("."))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Writer {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct QuantizationTransform {
    pub scale: [f64; 4],
    pub translate: [f64; 4],
}

impl From<&QuantizationTransform> for pbf::Transform {
    fn from(transform: &QuantizationTransform) -> Self {
        let [x_scale, y_scale, z_scale, m_scale] = transform.scale;
        let [x_translate, y_translate, z_translate, m_translate] = transform.translate;
        pbf::Transform {
            quantize_origin_postion: pbf::QuantizeOriginPostion::LowerLeft.into(),
            scale: Some(pbf::Scale {
                x_scale,
                y_scale,
                z_scale,
                m_scale,
            }),
            translate: Some(pbf::Translate {
                x_translate,
                y_translate,
                z_translate,
                m_translate,
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct MultiscaleLevel {
    pub column: ColumnPath,
    pub level: u8,
    pub resolution: f64,
    pub scale: f64,
    pub transform: QuantizationTransform,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GeometryType {
    Point,
    Multipoint,
    Polygon,
    Polyline,
}

impl From<GeometryType> for pbf::GeometryType {
    fn from(geometry_type: GeometryType) -> Self {
        match geometry_type {
            GeometryType::Point => pbf::GeometryType::EsriGeometryTypePoint,
            GeometryType::Multipoint => pbf::GeometryType::EsriGeometryTypeMultipoint,
            GeometryType::Polygon => pbf::GeometryType::EsriGeometryTypePolygon,
            GeometryType::Polyline => pbf::GeometryType::EsriGeometryTypePolyline,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum Encoding {
    #[serde(rename = "esriPBF")]
    EsriPbf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayMetadata {
    pub version: String,
    pub writer: Option<Writer>,
    pub full_extent: EsriEnvelope,
    pub wkid: Option<i32>,
    pub wkt: Option<String>,
    #[serde(default)]
    pub has_z: bool,
    #[serde(default)]
    pub has_m: bool,
}

impl From<&DisplayMetadata> for VertexDescription {
    fn from(display: &DisplayMetadata) -> Self {
        [
            display.has_z.then_some(Attribute::Z),
            display.has_m.then_some(Attribute::M),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XzMetadata {
    #[serde(flatten)]
    pub display: DisplayMetadata,
    pub geometry_type: GeometryType,
    pub max_level: u8,
    pub code: ColumnPath,
    pub levels: Vec<MultiscaleLevel>,
    pub encoding: Encoding,
}

impl XzMetadata {
    pub fn level(&self, resolution: f64) -> Result<&MultiscaleLevel, Error> {
        if !(resolution.is_finite() && resolution > 0.0) {
            return Err(Error::Resolution(resolution));
        }
        let mut levels = self.levels.iter().collect::<Vec<_>>();
        levels.sort_by_key(|level| level.level);
        let tolerance = 2f64.powf(0.001);
        levels
            .iter()
            .find(|level| level.resolution <= resolution * tolerance)
            .or(levels.last())
            .copied()
            .ok_or(Error::NoLevels)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZMetadata {
    #[serde(flatten)]
    pub display: DisplayMetadata,
    pub code: ColumnPath,
    pub x_column: ColumnPath,
    pub y_column: ColumnPath,
    pub z_column: Option<ColumnPath>,
    pub m_column: Option<ColumnPath>,
    pub coordinate_precision: u8,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum GeodisplayMetadata {
    Xz(XzMetadata),
    Z(ZMetadata),
}

const ROOT_KEY: &str = "geodisplay";
const SPARK_KEY: &str = "org.apache.spark.sql.parquet.row.metadata";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeodisplayJson {
    pub json: String,
    pub group: Option<String>,
}

impl TryFrom<&FileMetaData> for GeodisplayJson {
    type Error = Error;

    fn try_from(file: &FileMetaData) -> Result<Self, Self::Error> {
        let value = |key: &str| {
            file.key_value_metadata()
                .into_iter()
                .flatten()
                .find(|kv| kv.key == key)
                .and_then(|kv| kv.value.as_deref())
        };
        let root = value(ROOT_KEY).map(|json| GeodisplayJson {
            json: json.to_string(),
            group: None,
        });
        let fields = value(SPARK_KEY)
            .map(serde_json::from_str::<SparkSchema>)
            .transpose()?
            .map(|schema| schema.fields)
            .unwrap_or_default();
        let mut indexes = fields.into_iter().filter_map(|field| {
            let metadata = field.metadata?;
            let tagged = serde_json::from_str::<Tagged>(metadata.get()).ok()?;
            matches!(tagged.r#type.as_deref(), Some("xz" | "z")).then(|| GeodisplayJson {
                json: metadata.get().to_string(),
                group: Some(field.name),
            })
        });
        let spark = indexes.next();
        if indexes.next().is_some() {
            return Err(Error::MultipleIndexes);
        }
        match (root, spark) {
            (Some(_), Some(_)) => Err(Error::BothMetadataForms),
            (Some(json), None) | (None, Some(json)) => Ok(json),
            (None, None) => Err(Error::MissingMetadata),
        }
    }
}

impl TryFrom<&GeodisplayJson> for GeodisplayMetadata {
    type Error = Error;

    fn try_from(raw: &GeodisplayJson) -> Result<Self, Self::Error> {
        let mut metadata = serde_json::from_str::<GeodisplayMetadata>(&raw.json)?;
        let Some(group) = &raw.group else {
            return Ok(metadata);
        };
        match &mut metadata {
            GeodisplayMetadata::Xz(xz) => {
                xz.code.nest(group);
                for level in &mut xz.levels {
                    level.column.nest(group);
                }
            }
            GeodisplayMetadata::Z(z) => {
                for path in [&mut z.code, &mut z.x_column, &mut z.y_column] {
                    path.nest(group);
                }
                for path in [&mut z.z_column, &mut z.m_column].into_iter().flatten() {
                    path.nest(group);
                }
            }
        }
        Ok(metadata)
    }
}

#[derive(Deserialize)]
struct SparkSchema {
    fields: Vec<SparkField>,
}

#[derive(Deserialize)]
struct SparkField {
    name: String,
    metadata: Option<Box<RawValue>>,
}

#[derive(Deserialize)]
struct Tagged {
    r#type: Option<String>,
}

#[cfg(test)]
mod tests;
