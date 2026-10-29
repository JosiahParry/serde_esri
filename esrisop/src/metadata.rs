use crate::{error::Error, xz::MaxLevel};
use arrow_array::{cast::AsArray, types::Float64Type, ArrayRef, Float64Array, RecordBatch};
use esripbf::esri_p_buffer::feature_collection_p_buffer as pbf;
use parquet::{
    file::metadata::FileMetaData,
    schema::types::{ColumnPath, SchemaDescriptor},
};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use serde_esri::{
    enginex::{
        description::{Attribute, VertexDescription},
        flags::GeometryType,
    },
    geometry::EsriEnvelope,
};
use serde_json::{value::RawValue, Value};

fn column_path<'de, D: Deserializer<'de>>(deserializer: D) -> Result<ColumnPath, D::Error> {
    let parts = match Value::deserialize(deserializer)? {
        Value::String(name) => vec![name],
        value => serde_json::from_value::<Vec<String>>(value).map_err(de::Error::custom)?,
    };
    Ok(ColumnPath::new(parts))
}

fn optional_column_path<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ColumnPath>, D::Error> {
    column_path(deserializer).map(Some)
}

fn serialize_column_path<S: Serializer>(
    path: &ColumnPath,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match path.parts() {
        [name] => serializer.serialize_str(name),
        parts => parts.serialize(serializer),
    }
}

fn serialize_optional_column_path<S: Serializer>(
    path: &Option<ColumnPath>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match path {
        Some(path) => serialize_column_path(path, serializer),
        None => serializer.serialize_none(),
    }
}

fn optional_geometry_type<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<GeometryType>, D::Error> {
    match String::deserialize(deserializer)?.as_str() {
        "point" => Ok(Some(GeometryType::Point)),
        "multipoint" => Ok(Some(GeometryType::MultiPoint)),
        "polyline" => Ok(Some(GeometryType::Polyline)),
        "polygon" => Ok(Some(GeometryType::Polygon)),
        other => Err(de::Error::unknown_variant(
            other,
            &["point", "multipoint", "polyline", "polygon"],
        )),
    }
}

fn serialize_optional_geometry_type<S: Serializer>(
    geometry_type: &Option<GeometryType>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match geometry_type {
        Some(GeometryType::Point) => serializer.serialize_str("point"),
        Some(GeometryType::MultiPoint) => serializer.serialize_str("multipoint"),
        Some(GeometryType::Polyline) => serializer.serialize_str("polyline"),
        Some(GeometryType::Polygon) => serializer.serialize_str("polygon"),
        Some(other) => Err(ser::Error::custom(format!(
            "geodisplay has no geometryType for {other:?}"
        ))),
        None => serializer.serialize_none(),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Writer {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MultiscaleLevel {
    #[serde(
        deserialize_with = "column_path",
        serialize_with = "serialize_column_path"
    )]
    pub column: ColumnPath,
    pub level: u8,
    pub resolution: f64,
    pub scale: f64,
    pub transform: QuantizationTransform,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Encoding {
    #[serde(rename = "esriPBF", alias = "esri-pbf")]
    EsriPbf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayMetadata {
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer: Option<Writer>,
    pub full_extent: EsriEnvelope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wkid: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wkt: Option<String>,
    #[serde(
        default,
        deserialize_with = "optional_geometry_type",
        serialize_with = "serialize_optional_geometry_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub geometry_type: Option<GeometryType>,
    #[serde(default)]
    pub has_z: bool,
    #[serde(default)]
    pub has_m: bool,
    #[serde(skip)]
    pub json: String,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XzMetadata {
    #[serde(flatten)]
    pub display: DisplayMetadata,
    pub max_level: MaxLevel,
    #[serde(
        rename = "code",
        deserialize_with = "column_path",
        serialize_with = "serialize_column_path"
    )]
    pub index_column: ColumnPath,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZMetadata {
    #[serde(flatten)]
    pub display: DisplayMetadata,
    #[serde(
        rename = "code",
        deserialize_with = "column_path",
        serialize_with = "serialize_column_path"
    )]
    pub index_column: ColumnPath,
    #[serde(
        deserialize_with = "column_path",
        serialize_with = "serialize_column_path"
    )]
    pub x_column: ColumnPath,
    #[serde(
        deserialize_with = "column_path",
        serialize_with = "serialize_column_path"
    )]
    pub y_column: ColumnPath,
    #[serde(
        default,
        deserialize_with = "optional_column_path",
        serialize_with = "serialize_optional_column_path",
        skip_serializing_if = "Option::is_none"
    )]
    pub z_column: Option<ColumnPath>,
    #[serde(
        default,
        deserialize_with = "optional_column_path",
        serialize_with = "serialize_optional_column_path",
        skip_serializing_if = "Option::is_none"
    )]
    pub m_column: Option<ColumnPath>,
    pub coordinate_precision: u8,
}

impl ZMetadata {
    pub fn index_key(&self, x: f64, y: f64) -> i64 {
        let bits = u32::from(self.coordinate_precision.min(31));
        let extent = &self.display.full_extent;
        let cells = 1_i64 << bits;
        let cell = |value: f64, min: f64, max: f64| {
            if bits == 0 || (max - min).abs() < f64::EPSILON {
                return 0;
            }
            (((value - min) / (max - min) * cells as f64) as i64).clamp(0, cells - 1) as u32
        };
        let key = zorder::index_of([
            cell(x, extent.xmin, extent.xmax),
            cell(y, extent.ymin, extent.ymax),
        ]);
        key as i64
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum GeodisplayMetadata {
    Xz(XzMetadata),
    Z(ZMetadata),
}

impl GeodisplayMetadata {
    pub fn display(&self) -> &DisplayMetadata {
        match self {
            GeodisplayMetadata::Xz(xz) => &xz.display,
            GeodisplayMetadata::Z(z) => &z.display,
        }
    }

    /// The index columns a read leaves out unless asked: XZ codes and levels, and Z columns
    /// inside a group, but not a Z index's top-level x and y.
    pub fn index_columns(&self) -> Vec<&ColumnPath> {
        match self {
            GeodisplayMetadata::Xz(xz) => std::iter::once(&xz.index_column)
                .chain(xz.levels.iter().map(|level| &level.column))
                .collect(),
            GeodisplayMetadata::Z(z) => [
                Some(&z.index_column),
                Some(&z.x_column),
                Some(&z.y_column),
                z.z_column.as_ref(),
                z.m_column.as_ref(),
            ]
            .into_iter()
            .flatten()
            .filter(|path| path.parts().len() > 1 || *path == &z.index_column)
            .collect(),
        }
    }
}

pub(crate) trait ColumnLookup {
    fn leaf(&self, schema: &SchemaDescriptor) -> Result<usize, Error>;
    fn column<'a>(&self, batch: &'a RecordBatch) -> Result<&'a ArrayRef, Error>;
    fn doubles<'a>(&self, batch: &'a RecordBatch) -> Result<&'a Float64Array, Error>;
    fn binaries<'a>(&self, batch: &'a RecordBatch) -> Result<Vec<Option<&'a [u8]>>, Error>;
}

impl ColumnLookup for ColumnPath {
    fn leaf(&self, schema: &SchemaDescriptor) -> Result<usize, Error> {
        schema
            .columns()
            .iter()
            .position(|c| c.path() == self)
            .ok_or_else(|| Error::MissingColumn(self.to_string()))
    }

    fn column<'a>(&self, batch: &'a RecordBatch) -> Result<&'a ArrayRef, Error> {
        let missing = || Error::MissingColumn(self.to_string());
        let (root, children) = self.parts().split_first().ok_or_else(missing)?;
        children.iter().try_fold(
            batch.column_by_name(root).ok_or_else(missing)?,
            |array, name| {
                array
                    .as_struct_opt()
                    .and_then(|group| group.column_by_name(name))
                    .ok_or_else(missing)
            },
        )
    }

    fn doubles<'a>(&self, batch: &'a RecordBatch) -> Result<&'a Float64Array, Error> {
        self.column(batch)?
            .as_primitive_opt::<Float64Type>()
            .ok_or_else(|| Error::ColumnType {
                column: self.to_string(),
                expected: "double",
            })
    }

    fn binaries<'a>(&self, batch: &'a RecordBatch) -> Result<Vec<Option<&'a [u8]>>, Error> {
        let values = self.column(batch)?;
        match (values.as_binary_opt::<i32>(), values.as_binary_opt::<i64>()) {
            (Some(values), _) => Ok(values.iter().collect()),
            (None, Some(values)) => Ok(values.iter().collect()),
            (None, None) => Ok(values
                .as_binary_view_opt()
                .ok_or_else(|| Error::ColumnType {
                    column: self.to_string(),
                    expected: "binary",
                })?
                .iter()
                .collect()),
        }
    }
}

const ROOT_KEY: &str = "geodisplay";
const SPARK_KEY: &str = "org.apache.spark.sql.parquet.row.metadata";

impl TryFrom<&FileMetaData> for GeodisplayMetadata {
    type Error = Error;

    fn try_from(file: &FileMetaData) -> Result<Self, Self::Error> {
        let value = |key: &str| {
            file.key_value_metadata()
                .into_iter()
                .flatten()
                .find(|kv| kv.key == key)
                .and_then(|kv| kv.value.as_deref())
        };
        let fields = value(SPARK_KEY)
            .map(serde_json::from_str::<SparkSchema>)
            .transpose()?
            .map(|schema| schema.fields)
            .unwrap_or_default();
        let mut indexes = fields.into_iter().filter_map(|field| {
            let metadata = field.metadata?;
            let tagged = serde_json::from_str::<Tagged>(metadata.get()).ok()?;
            matches!(tagged.r#type.as_deref(), Some("xz" | "z"))
                .then(|| (metadata.get().to_string(), Some(field.name)))
        });
        let spark = indexes.next();
        if indexes.next().is_some() {
            return Err(Error::MultipleIndexes);
        }
        let (json, group) = match (value(ROOT_KEY), spark) {
            (Some(_), Some(_)) => return Err(Error::BothMetadataForms),
            (Some(json), None) => (json.to_string(), None),
            (None, Some(spark)) => spark,
            (None, None) => return Err(Error::MissingMetadata),
        };

        let mut metadata = serde_json::from_str::<GeodisplayMetadata>(&json)?;
        let (display, paths) = match &mut metadata {
            GeodisplayMetadata::Xz(xz) => (
                &mut xz.display,
                std::iter::once(&mut xz.index_column)
                    .chain(xz.levels.iter_mut().map(|level| &mut level.column))
                    .collect::<Vec<_>>(),
            ),
            GeodisplayMetadata::Z(z) => (
                &mut z.display,
                [&mut z.index_column, &mut z.x_column, &mut z.y_column]
                    .into_iter()
                    .chain([&mut z.z_column, &mut z.m_column].into_iter().flatten())
                    .collect(),
            ),
        };
        if let Some(group) = group {
            for path in paths {
                let parts = std::iter::once(group.clone())
                    .chain(path.parts().iter().cloned())
                    .collect();
                *path = ColumnPath::new(parts);
            }
        }
        display.json = json;
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
