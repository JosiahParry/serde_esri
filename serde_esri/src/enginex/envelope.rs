//! Envelopes over every vertex attribute, as the engine's `queryEnvelope` computes them.
//! Curves contribute their end points only, not their extent between them.

use crate::enginex::{Envelope, Envelope2D, Geometry, Interval, Vertex};

/// Bounds of x/y and of each attribute any vertex carries; `NaN` values are skipped.
impl FromIterator<Vertex> for Envelope {
    fn from_iter<I: IntoIterator<Item = Vertex>>(vertices: I) -> Self {
        let mut envelope = Envelope::default();
        for v in vertices {
            envelope.xy = Some(match envelope.xy {
                Some(e) => Envelope2D {
                    xmin: e.xmin.min(v.x),
                    ymin: e.ymin.min(v.y),
                    xmax: e.xmax.max(v.x),
                    ymax: e.ymax.max(v.y),
                },
                None => Envelope2D {
                    xmin: v.x,
                    ymin: v.y,
                    xmax: v.x,
                    ymax: v.y,
                },
            });
            let widen = |i: Option<Interval<f64>>, value: f64| match i {
                Some(i) => Interval {
                    min: i.min.min(value),
                    max: i.max.max(value),
                },
                None => Interval {
                    min: value,
                    max: value,
                },
            };
            if let Some(z) = v.z {
                envelope.z = Some(widen(envelope.z, z));
            }
            if let Some(m) = v.m {
                envelope.m = Some(widen(envelope.m, m));
            }
            if let Some(id) = v.id {
                envelope.id = Some(match envelope.id {
                    Some(i) => Interval {
                        min: i.min.min(id),
                        max: i.max.max(id),
                    },
                    None => Interval { min: id, max: id },
                });
            }
        }
        envelope
    }
}

impl Geometry {
    /// The envelope of every vertex; empty geometries have an empty envelope.
    pub fn envelope(&self) -> Envelope {
        match self {
            Geometry::Point(p) => p.0.into_iter().collect(),
            Geometry::Line(l) => [l.start, l.end].into_iter().collect(),
            Geometry::Envelope(e) => *e,
            Geometry::MultiPoint(mp) => mp.vertices.iter().collect(),
            Geometry::Polyline(p) => p.0.vertices.iter().collect(),
            Geometry::Polygon(p) => p.rings.vertices.iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests;
