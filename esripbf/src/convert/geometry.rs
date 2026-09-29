//! Decodes quantized, delta-encoded geometries into engine and Esri JSON geometries.
//!
//! Each vertex holds x and y as deltas from the previous vertex in its part, then Z and M, when
//! present, as plain integers. The transform scales and translates them; an upper-left origin flips y.

use crate::{
    convert::FromPbfError,
    esri_p_buffer::feature_collection_p_buffer::{self as pbf, GeometryType, QuantizeOriginPostion},
};
#[cfg(feature = "geoarrow")]
use serde_esri::enginex::geometry::{Geometry, MultiPoint, Point, Polygon, Polyline};
use serde_esri::{
    enginex::{geometry::MultiPath, vertex::VertexAttributes},
    geometry::{
        EsriCoord, EsriGeometry, EsriLineString, EsriMultiPoint, EsriPoint, EsriPolygon,
        EsriPolyline,
    },
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

/// Whether a part's last vertex repeats its first to close it, which the engine leaves implicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parts {
    Paths,
    Rings,
}

/// The transform of one `FeatureResult`, with an axis per ordinate.
#[derive(Clone, Debug)]
pub(super) struct Quantization {
    x: Axis,
    y: Axis,
    z: Option<Axis>,
    m: Option<Axis>,
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
        Ok(Quantization {
            x: Axis {
                scale: scale.x_scale,
                translate: translate.x_translate,
            },
            y: Axis {
                scale: y_scale,
                translate: translate.y_translate,
            },
            z: result.has_z.then_some(Axis {
                scale: scale.z_scale,
                translate: translate.z_translate,
            }),
            m: result.has_m.then_some(Axis {
                scale: scale.m_scale,
                translate: translate.m_translate,
            }),
        })
    }
}

impl Quantization {
    fn width(&self) -> usize {
        2 + usize::from(self.z.is_some()) + usize::from(self.m.is_some())
    }

    /// The vertices split into paths by `lengths`, one path when there are none.
    fn multi_path(
        &self,
        geometry: &pbf::Geometry,
        parts: Parts,
    ) -> Result<MultiPath, FromPbfError> {
        let width = self.width();
        let lengths = match geometry.lengths.as_slice() {
            [] => vec![geometry.coords.len() / width],
            lengths => lengths.iter().map(|n| *n as usize).collect(),
        };
        let expected = lengths.iter().sum::<usize>() * width;
        if expected != geometry.coords.len() {
            return Err(FromPbfError::Coordinates {
                expected,
                found: geometry.coords.len(),
            });
        }
        let count = expected / width;
        let mut vertices = VertexAttributes {
            xy: Vec::with_capacity(count),
            z: self.z.map(|_| Vec::with_capacity(count)),
            m: self.m.map(|_| Vec::with_capacity(count)),
            id: None,
        };
        let mut path_offsets = Vec::with_capacity(lengths.len() + 1);
        path_offsets.push(0);
        let mut coords = geometry.coords.chunks_exact(width);
        for length in lengths {
            let start = vertices.xy.len();
            let (mut x, mut y) = (0_i64, 0_i64);
            for delta in coords.by_ref().take(length) {
                x += delta[0];
                y += delta[1];
                vertices.xy.push([self.x.apply(x), self.y.apply(y)]);
                if let (Some(zs), Some(axis)) = (vertices.z.as_mut(), self.z) {
                    zs.push(axis.apply(delta[2]));
                }
                if let (Some(ms), Some(axis)) = (vertices.m.as_mut(), self.m) {
                    ms.push(axis.apply(delta[width - 1]));
                }
            }
            let end = vertices.xy.len();
            let closes = parts == Parts::Rings
                && end > start + 1
                && vertices.get(start) == vertices.get(end - 1);
            if closes {
                vertices.xy.pop();
                vertices.z.as_mut().map(Vec::pop);
                vertices.m.as_mut().map(Vec::pop);
            }
            let offset = i32::try_from(vertices.xy.len()).map_err(|_| FromPbfError::TooLarge)?;
            path_offsets.push(offset);
        }
        Ok(MultiPath {
            path_flags: vec![Default::default(); path_offsets.len() - 1],
            vertices,
            path_offsets,
            segments: None,
        })
    }

    #[cfg(feature = "geoarrow")]
    pub(super) fn engine(
        &self,
        geometry_type: GeometryType,
        geometry: &pbf::Geometry,
    ) -> Result<Geometry, FromPbfError> {
        Ok(match geometry_type {
            GeometryType::EsriGeometryTypePoint => {
                let paths = self.multi_path(geometry, Parts::Paths)?;
                Geometry::Point(Point(paths.vertices.get(0)))
            }
            GeometryType::EsriGeometryTypeMultipoint => {
                let paths = self.multi_path(geometry, Parts::Paths)?;
                Geometry::MultiPoint(MultiPoint {
                    vertices: paths.vertices,
                })
            }
            GeometryType::EsriGeometryTypePolyline => {
                Geometry::Polyline(Polyline(self.multi_path(geometry, Parts::Paths)?))
            }
            GeometryType::EsriGeometryTypePolygon => {
                Geometry::Polygon(Polygon::from(self.multi_path(geometry, Parts::Rings)?))
            }
            GeometryType::EsriGeometryTypeMultipatch => {
                return Err(FromPbfError::UnsupportedGeometry("multipatch"))
            }
            GeometryType::EsriGeometryTypeEnvelope => {
                return Err(FromPbfError::UnsupportedGeometry("envelope"))
            }
            GeometryType::EsriGeometryTypeNone => {
                return Err(FromPbfError::UnsupportedGeometry("none"))
            }
        })
    }

    /// Each part's coordinates as `[x, y, z?, m?]`, rings keeping their closing vertex.
    fn parts<const N: usize>(
        &self,
        geometry: &pbf::Geometry,
    ) -> Result<Vec<Vec<EsriCoord<N>>>, FromPbfError> {
        let paths = self.multi_path(geometry, Parts::Paths)?;
        let coord = |i: usize| {
            let mut coord = [f64::NAN; N];
            if let Some(v) = paths.vertices.get(i) {
                let ordinates = [Some(v.x), Some(v.y), v.z, v.m].into_iter().flatten();
                for (slot, value) in coord.iter_mut().zip(ordinates) {
                    *slot = value;
                }
            }
            EsriCoord(coord)
        };
        Ok(paths
            .path_offsets
            .windows(2)
            .map(|w| (w[0] as usize..w[1] as usize).map(coord).collect())
            .collect())
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
                let (z, m) = match (self.z, self.m) {
                    (Some(_), Some(_)) => (at(2), at(3)),
                    (Some(_), None) => (at(2), None),
                    (None, _) => (None, at(2)),
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
