//! Decodes quantized, delta-encoded geometries into Esri JSON geometries.
//!
//! Each vertex holds x and y as deltas from the previous vertex in its part, then Z and M, when
//! present, as plain integers. The transform scales and translates them; an upper-left origin flips y.

use crate::{
    convert::FromPbfError,
    feature_collection_p_buffer::{self as pbf, GeometryType, QuantizeOriginPostion},
};
use serde_esri::geometry::{
    EsriCoord, EsriGeometry, EsriLineString, EsriMultiPoint, EsriPoint, EsriPolygon, EsriPolyline,
};

#[derive(Clone, Copy, Debug)]
struct Axis {
    scale: f64,
    translate: f64,
}

impl Axis {
    fn apply(self, quantized: i64) -> f64 {
        quantized as f64 * self.scale + self.translate
    }
}

/// The transform of one `FeatureResult`, with an axis per ordinate in coordinate order.
#[derive(Clone, Debug)]
pub(super) struct Quantization {
    axes: Vec<Axis>,
    has_z: bool,
}

impl TryFrom<&pbf::FeatureResult> for Quantization {
    type Error = FromPbfError;

    fn try_from(result: &pbf::FeatureResult) -> Result<Self, Self::Error> {
        let transform = result
            .transform
            .as_ref()
            .ok_or(FromPbfError::MissingTransform)?;
        let scale = transform.scale.clone().unwrap_or_default();
        let translate = transform.translate.clone().unwrap_or_default();
        let y_scale = match transform.quantize_origin_postion() {
            QuantizeOriginPostion::UpperLeft => -scale.y_scale,
            QuantizeOriginPostion::LowerLeft => scale.y_scale,
        };
        let mut axes = vec![
            Axis {
                scale: scale.x_scale,
                translate: translate.x_translate,
            },
            Axis {
                scale: y_scale,
                translate: translate.y_translate,
            },
        ];
        if result.has_z {
            axes.push(Axis {
                scale: scale.z_scale,
                translate: translate.z_translate,
            });
        }
        if result.has_m {
            axes.push(Axis {
                scale: scale.m_scale,
                translate: translate.m_translate,
            });
        }
        Ok(Quantization {
            axes,
            has_z: result.has_z,
        })
    }
}

impl Quantization {
    /// Splits the coordinates into parts by `lengths`, one part when there are none.
    fn parts<const N: usize>(
        &self,
        geometry: &pbf::Geometry,
    ) -> Result<Vec<Vec<EsriCoord<N>>>, FromPbfError> {
        let lengths = match geometry.lengths.as_slice() {
            [] => vec![geometry.coords.len() / N],
            lengths => lengths.iter().map(|n| *n as usize).collect(),
        };
        let expected = lengths.iter().sum::<usize>() * N;
        if expected != geometry.coords.len() {
            return Err(FromPbfError::Coordinates {
                expected,
                found: geometry.coords.len(),
            });
        }
        let mut coords = geometry.coords.chunks_exact(N);
        let parts = lengths
            .into_iter()
            .map(|length| {
                let mut position = [0_i64; N];
                coords
                    .by_ref()
                    .take(length)
                    .map(|delta| {
                        let mut coord = [0.0; N];
                        for (i, axis) in self.axes.iter().enumerate() {
                            position[i] = if i < 2 {
                                position[i] + delta[i]
                            } else {
                                delta[i]
                            };
                            coord[i] = axis.apply(position[i]);
                        }
                        EsriCoord(coord)
                    })
                    .collect()
            })
            .collect();
        Ok(parts)
    }

    pub(super) fn geometry<const N: usize>(
        &self,
        geometry_type: GeometryType,
        geometry: &pbf::Geometry,
    ) -> Result<EsriGeometry<N>, FromPbfError> {
        let parts = self.parts::<N>(geometry)?;
        let geometry = match geometry_type {
            GeometryType::EsriGeometryTypePoint => {
                let coord = parts.first().and_then(|part| part.first());
                let at = |i: usize| coord.and_then(|c| c.0.get(i).copied());
                let (z, m) = match (self.has_z, N) {
                    (true, 4) => (at(2), at(3)),
                    (true, _) => (at(2), None),
                    (false, _) => (None, at(2)),
                };
                EsriGeometry::Point(EsriPoint {
                    x: at(0).unwrap_or(f64::NAN),
                    y: at(1).unwrap_or(f64::NAN),
                    z,
                    m,
                    spatial_reference: None,
                })
            }
            GeometryType::EsriGeometryTypeMultipoint => EsriGeometry::MultiPoint(EsriMultiPoint {
                has_z: None,
                has_m: None,
                points: parts.into_iter().flatten().collect(),
                spatial_reference: None,
            }),
            GeometryType::EsriGeometryTypePolyline => EsriGeometry::Polyline(EsriPolyline {
                has_z: None,
                has_m: None,
                paths: parts.into_iter().map(EsriLineString).collect(),
                spatial_reference: None,
            }),
            GeometryType::EsriGeometryTypePolygon => EsriGeometry::Polygon(EsriPolygon {
                has_z: None,
                has_m: None,
                rings: parts.into_iter().map(EsriLineString).collect(),
                spatial_reference: None,
            }),
            GeometryType::EsriGeometryTypeMultipatch => {
                return Err(FromPbfError::UnsupportedGeometry("multipatch"))
            }
            GeometryType::EsriGeometryTypeEnvelope => {
                return Err(FromPbfError::UnsupportedGeometry("envelope"))
            }
            GeometryType::EsriGeometryTypeNone => {
                return Err(FromPbfError::UnsupportedGeometry("none"))
            }
        };
        Ok(geometry)
    }
}
