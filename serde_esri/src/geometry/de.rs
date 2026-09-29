//! Deserializes an [`EsriGeometry`] in one pass, choosing the variant by its keys: `rings` is a
//! polygon, `paths` a polyline, `points` a multipoint, `xmin` an envelope, and `x` a point.
//! An untagged enum would buffer each geometry and retry it against every variant.

use crate::{
    geometry::{
        EsriCoord, EsriEnvelope, EsriGeometry, EsriLineString, EsriMultiPoint, EsriPoint,
        EsriPolygon, EsriPolyline,
    },
    spatial_reference::SpatialReference,
};
use serde::{de, Deserialize, Deserializer};

/// Every key an Esri geometry object can hold.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnyGeometry<const N: usize> {
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    m: Option<f64>,
    points: Option<Vec<EsriCoord<N>>>,
    paths: Option<Vec<EsriLineString<N>>>,
    rings: Option<Vec<EsriLineString<N>>>,
    xmin: Option<f64>,
    ymin: Option<f64>,
    xmax: Option<f64>,
    ymax: Option<f64>,
    zmin: Option<f64>,
    zmax: Option<f64>,
    mmin: Option<f64>,
    mmax: Option<f64>,
    has_z: Option<bool>,
    has_m: Option<bool>,
    spatial_reference: Option<SpatialReference>,
}

impl<'de, const N: usize> Deserialize<'de> for EsriGeometry<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let g = AnyGeometry::<N>::deserialize(deserializer)?;
        let (has_z, has_m, spatial_reference) = (g.has_z, g.has_m, g.spatial_reference);
        if let Some(rings) = g.rings {
            return Ok(EsriGeometry::Polygon(EsriPolygon {
                has_z,
                has_m,
                rings,
                spatial_reference,
            }));
        }
        if let Some(paths) = g.paths {
            return Ok(EsriGeometry::Polyline(EsriPolyline {
                has_z,
                has_m,
                paths,
                spatial_reference,
            }));
        }
        if let Some(points) = g.points {
            return Ok(EsriGeometry::MultiPoint(EsriMultiPoint {
                has_z,
                has_m,
                points,
                spatial_reference,
            }));
        }
        if let (Some(xmin), Some(ymin), Some(xmax), Some(ymax)) = (g.xmin, g.ymin, g.xmax, g.ymax) {
            return Ok(EsriGeometry::Envelope(EsriEnvelope {
                xmin,
                ymin,
                xmax,
                ymax,
                zmin: g.zmin,
                zmax: g.zmax,
                mmin: g.mmin,
                mmax: g.mmax,
                spatial_reference,
            }));
        }
        if let (Some(x), Some(y)) = (g.x, g.y) {
            return Ok(EsriGeometry::Point(EsriPoint {
                x,
                y,
                z: g.z,
                m: g.m,
                spatial_reference,
            }));
        }
        Err(de::Error::custom(
            "expected an Esri geometry with rings, paths, points, xmin, or x",
        ))
    }
}

#[cfg(test)]
mod tests;
