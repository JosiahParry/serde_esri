use crate::{
    error::Error,
    metadata::{ColumnLookup, GeodisplayMetadata},
    query::Query,
};
use arrow_array::{
    builder::BinaryBuilder, Array, ArrayRef, BooleanArray, Float64Array, RecordBatch,
};
use arrow_schema::{DataType, Field};
use esripbf::{convert::geometry::Quantization, esri_p_buffer::feature_collection_p_buffer as pbf};
use parquet::schema::types::ColumnPath;
use prost::Message;
use serde_esri::{
    enginex::{
        description::VertexDescription,
        geometry::{Envelope2D, Geometry, Point},
        shape::writer::EsriShapeBuffer,
        vertex::Vertex,
    },
    spatial_reference::SpatialReference,
};
use std::{collections::HashMap, sync::Arc};

enum Source {
    Level {
        level_column: ColumnPath,
        quantization: Quantization,
        geometry_type: pbf::GeometryType,
        extents: Vec<Envelope2D>,
    },
    Points {
        x: ColumnPath,
        y: ColumnPath,
        z: Option<ColumnPath>,
        m: Option<ColumnPath>,
    },
}

pub struct ShapeColumn {
    source: Source,
    field: Field,
}

impl ShapeColumn {
    pub fn try_new(geodisplay: &GeodisplayMetadata, query: &Query) -> Result<Option<Self>, Error> {
        let source = match geodisplay {
            GeodisplayMetadata::Xz(xz) => {
                let Some(resolution) = query.resolution else {
                    return Ok(None);
                };
                let level = xz.level(resolution)?;
                Source::Level {
                    level_column: level.column.clone(),
                    quantization: Quantization::new(
                        &pbf::Transform::from(&level.transform),
                        VertexDescription::from(&xz.display),
                    ),
                    geometry_type: pbf::GeometryType::try_from(
                        xz.display.geometry_type.ok_or(Error::MissingGeometryType)?,
                    )?,
                    extents: query.extents.clone(),
                }
            }
            GeodisplayMetadata::Z(z) => Source::Points {
                x: z.x_column.clone(),
                y: z.y_column.clone(),
                z: z.z_column.clone().filter(|_| z.display.has_z),
                m: z.m_column.clone().filter(|_| z.display.has_m),
            },
        };
        let display = geodisplay.display();
        let spatial_reference = match (&display.wkt, display.wkid) {
            (Some(wkt), _) => wkt.clone(),
            (None, Some(wkid)) => serde_json::to_string(&SpatialReference {
                wkid: Some(wkid),
                latest_wkid: None,
                vcs_wkid: None,
                latest_vcs_wkid: None,
                wkt: None,
                wkt2: None,
            })?,
            (None, None) => String::new(),
        };
        let metadata = HashMap::from([
            ("esri.encoding".to_string(), "EsriShape".to_string()),
            ("esri.sr_wkt".to_string(), spatial_reference),
            ("geodisplay".to_string(), display.json.clone()),
        ]);
        Ok(Some(ShapeColumn {
            source,
            field: Field::new("geometry", DataType::Binary, true).with_metadata(metadata),
        }))
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn paths(&self) -> Vec<&ColumnPath> {
        match &self.source {
            Source::Level { level_column, .. } => vec![level_column],
            Source::Points { x, y, z, m } => [Some(x), Some(y), z.as_ref(), m.as_ref()]
                .into_iter()
                .flatten()
                .collect(),
        }
    }

    pub fn build(&self, batch: &RecordBatch) -> Result<(ArrayRef, Option<BooleanArray>), Error> {
        let rows = batch.num_rows();
        match &self.source {
            Source::Level {
                level_column,
                quantization,
                geometry_type,
                extents,
            } => {
                let bytes = level_column.binaries(batch)?;
                let mut shapes = BinaryBuilder::with_capacity(rows, 0);
                let mut hits = Vec::with_capacity(rows);
                for bytes in bytes {
                    let message = bytes.map(pbf::Geometry::decode).transpose()?;
                    let geometry = message
                        .filter(|message| !message.coords.is_empty())
                        .map(|message| quantization.engine(*geometry_type, &message))
                        .transpose()?;
                    let Some(geometry) = geometry else {
                        shapes.append_null();
                        hits.push(false);
                        continue;
                    };
                    shapes.append_value(EsriShapeBuffer::try_from(&geometry)?.0);
                    hits.push(geometry.envelope().xy.is_some_and(|b| {
                        extents.iter().any(|e| {
                            b.xmin <= e.xmax
                                && b.xmax >= e.xmin
                                && b.ymin <= e.ymax
                                && b.ymax >= e.ymin
                        })
                    }));
                }
                let keep = (!extents.is_empty()).then(|| BooleanArray::from(hits));
                Ok((Arc::new(shapes.finish()), keep))
            }
            Source::Points { x, y, z, m } => {
                let (xs, ys) = (x.doubles(batch)?, y.doubles(batch)?);
                let zs = z.as_ref().map(|z| z.doubles(batch)).transpose()?;
                let ms = m.as_ref().map(|m| m.doubles(batch)).transpose()?;
                let mut shapes = BinaryBuilder::with_capacity(rows, 0);
                for row in 0..rows {
                    let value = |values: &Float64Array| {
                        values
                            .is_valid(row)
                            .then(|| values.value(row))
                            .filter(|v| !v.is_nan())
                    };
                    let (Some(x), Some(y)) = (value(xs), value(ys)) else {
                        shapes.append_null();
                        continue;
                    };
                    let point = Geometry::Point(Point(Some(Vertex {
                        x,
                        y,
                        z: zs.map(|zs| value(zs).unwrap_or(f64::NAN)),
                        m: ms.map(|ms| value(ms).unwrap_or(f64::NAN)),
                        id: None,
                    })));
                    shapes.append_value(EsriShapeBuffer::try_from(&point)?.0);
                }
                Ok((Arc::new(shapes.finish()), None))
            }
        }
    }
}
