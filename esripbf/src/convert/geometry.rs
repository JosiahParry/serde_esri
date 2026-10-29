//! Decodes quantized, delta-encoded geometries into engine and Esri JSON geometries, and encodes
//! engine geometries back.
//!
//! Each vertex holds x and y as deltas from the previous vertex in its part, then Z and M, when
//! present, as plain integers. The transform scales and translates them; an upper-left origin flips y.

use crate::{
    convert::{FromPbfError, ToPbfError},
    esri_p_buffer::feature_collection_p_buffer::{
        self as pbf, GeometryType, QuantizeOriginPostion,
    },
};
use serde_esri::{
    enginex::{
        description::{Attribute, VertexDescription},
        flags,
        geometry::{Geometry, MultiPath, MultiPoint, Point, Polygon, Polyline},
        vertex::{Vertex, VertexAttributes},
    },
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

    fn quantize(self, value: f64) -> i64 {
        if value.is_finite() {
            ((value - self.translate) / self.scale).round() as i64
        } else {
            0
        }
    }
}

impl TryFrom<flags::GeometryType> for GeometryType {
    type Error = FromPbfError;

    fn try_from(geometry_type: flags::GeometryType) -> Result<Self, Self::Error> {
        match geometry_type {
            flags::GeometryType::Unknown => Ok(GeometryType::EsriGeometryTypeNone),
            flags::GeometryType::Point => Ok(GeometryType::EsriGeometryTypePoint),
            flags::GeometryType::MultiPoint => Ok(GeometryType::EsriGeometryTypeMultipoint),
            flags::GeometryType::Polyline => Ok(GeometryType::EsriGeometryTypePolyline),
            flags::GeometryType::Polygon => Ok(GeometryType::EsriGeometryTypePolygon),
            flags::GeometryType::Envelope => Ok(GeometryType::EsriGeometryTypeEnvelope),
            flags::GeometryType::Line => Err(FromPbfError::UnsupportedGeometry("line")),
            flags::GeometryType::Bezier => Err(FromPbfError::UnsupportedGeometry("bezier")),
            flags::GeometryType::EllipticArc => {
                Err(FromPbfError::UnsupportedGeometry("elliptic arc"))
            }
        }
    }
}

/// How a part holds its vertices. A ring repeats its first vertex to close, which the engine leaves
/// implicit. The encoder keeps every point, and drops repeated and collinear path and ring vertices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parts {
    Points,
    Paths,
    Rings,
}

/// The transform of one `FeatureResult`, with an axis per ordinate.
#[derive(Clone, Debug)]
pub struct Quantization {
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
        let description = [
            result.has_z.then_some(Attribute::Z),
            result.has_m.then_some(Attribute::M),
        ]
        .into_iter()
        .flatten()
        .collect();
        Ok(Quantization::new(transform, description))
    }
}

impl Quantization {
    pub fn new(transform: &pbf::Transform, description: VertexDescription) -> Self {
        let scale = transform.scale.clone().unwrap_or_default();
        let translate = transform.translate.clone().unwrap_or_default();
        let y_scale = match transform.quantize_origin_postion() {
            QuantizeOriginPostion::UpperLeft => -scale.y_scale,
            QuantizeOriginPostion::LowerLeft => scale.y_scale,
        };
        Quantization {
            x: Axis {
                scale: scale.x_scale,
                translate: translate.x_translate,
            },
            y: Axis {
                scale: y_scale,
                translate: translate.y_translate,
            },
            z: description.has(Attribute::Z).then_some(Axis {
                scale: scale.z_scale,
                translate: translate.z_translate,
            }),
            m: description.has(Attribute::M).then_some(Axis {
                scale: scale.m_scale,
                translate: translate.m_translate,
            }),
        }
    }

    /// The vertices split into paths by `lengths`, one path when there are none.
    fn multi_path(
        &self,
        geometry: &pbf::Geometry,
        parts: Parts,
    ) -> Result<MultiPath, FromPbfError> {
        let width = 2 + usize::from(self.z.is_some()) + usize::from(self.m.is_some());
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

    fn encode_part(
        &self,
        vertices: impl Iterator<Item = Vertex>,
        parts: Parts,
        encoded: &mut pbf::Geometry,
    ) {
        let flat = self.z.is_none() && self.m.is_none();
        let mut previous = None::<[i64; 4]>;
        let mut direction = [0_i64; 2];
        let mut length = 0_u32;
        for vertex in vertices {
            let quantized = [
                self.x.quantize(vertex.x),
                self.y.quantize(vertex.y),
                self.z
                    .map_or(0, |axis| axis.quantize(vertex.z.unwrap_or(f64::NAN))),
                self.m
                    .map_or(0, |axis| axis.quantize(vertex.m.unwrap_or(f64::NAN))),
            ];
            let delta = match previous {
                Some(last) => [quantized[0] - last[0], quantized[1] - last[1]],
                None => [quantized[0], quantized[1]],
            };
            let repeated = previous == Some(quantized);
            previous = Some(quantized);
            if parts != Parts::Points && repeated {
                continue;
            }
            let [dx, dy] = delta.map(i128::from);
            let [px, py] = direction.map(i128::from);
            let collinear = px * dy == dx * py && px * dx + py * dy > 0;
            if parts != Parts::Points && flat && collinear {
                if let [.., x, y] = encoded.coords.as_mut_slice() {
                    *x += delta[0];
                    *y += delta[1];
                }
                continue;
            }
            encoded.coords.extend_from_slice(&delta);
            if self.z.is_some() {
                encoded.coords.push(quantized[2]);
            }
            if self.m.is_some() {
                encoded.coords.push(quantized[3]);
            }
            if length > 0 {
                direction = delta;
            }
            length += 1;
        }
        if length > 0 {
            encoded.lengths.push(length);
        }
    }

    pub fn encode(&self, geometry: &Geometry) -> Result<pbf::Geometry, ToPbfError> {
        let mut encoded = pbf::Geometry::default();
        let (paths, parts) = match geometry {
            Geometry::Point(Point(vertex)) => {
                self.encode_part(vertex.iter().copied(), Parts::Points, &mut encoded);
                return Ok(encoded);
            }
            Geometry::MultiPoint(points) => {
                let vertices = (0..points.vertices.len()).filter_map(|i| points.vertices.get(i));
                self.encode_part(vertices, Parts::Points, &mut encoded);
                return Ok(encoded);
            }
            Geometry::Polyline(Polyline(paths)) => (paths, Parts::Paths),
            Geometry::Polygon(polygon) => (&polygon.rings, Parts::Rings),
            Geometry::Line(_) => return Err(ToPbfError::UnsupportedGeometry("line")),
            Geometry::Envelope(_) => return Err(ToPbfError::UnsupportedGeometry("envelope")),
        };
        if paths.has_nonlinear_segments() {
            return Err(ToPbfError::UnsupportedGeometry("curve"));
        }
        for path in 0..paths.path_count() {
            let range = paths.path_range(path).unwrap_or_default();
            let closing = (parts == Parts::Rings).then_some(range.start);
            let vertices = range.chain(closing).filter_map(|i| paths.vertices.get(i));
            self.encode_part(vertices, parts, &mut encoded);
        }
        Ok(encoded)
    }

    pub fn engine(
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

    pub(super) fn geometry<const N: usize>(
        &self,
        geometry_type: GeometryType,
        geometry: &pbf::Geometry,
    ) -> Result<EsriGeometry<N>, FromPbfError> {
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
        let parts = paths
            .path_offsets
            .windows(2)
            .map(|w| {
                (w[0] as usize..w[1] as usize)
                    .map(coord)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
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
