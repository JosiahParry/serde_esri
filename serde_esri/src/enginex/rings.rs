//! Ring areas and the engine's OGC view of polygons, in which each exterior ring is followed
//! by its holes and flagged with [`PathFlag::OgcStartPolygon`].

use crate::enginex::{FillRule, MultiPath, PathFlag, PathFlags, Polygon};
use std::ops::Range;

impl MultiPath {
    /// Signed area of a path as a closed ring, positive when clockwise (`calculateRingArea2D`).
    /// Curves count as straight segments between their end points.
    pub fn ring_area(&self, path_index: usize) -> Option<f64> {
        let ring = self.vertices.xy.get(self.path_range(path_index)?)?;
        let Some(&[x0, y0]) = ring.first() else {
            return Some(0.0);
        };
        let next = ring.iter().cycle().skip(1);
        Some(
            ring.iter()
                .zip(next)
                .map(|([x1, y1], [x2, y2])| ((x2 - x0) - (x1 - x0)) * ((y2 - y0) + (y1 - y0)) * 0.5)
                .sum(),
        )
    }

    /// Reverses a path's vertices in every attribute, keeping a closed path's first vertex
    /// first (`reversePath`).
    pub fn reverse_path(&mut self, path_index: usize) {
        let Some(range) = self.path_range(path_index) else {
            return;
        };
        let start = range.start + usize::from(self.is_closed_path(path_index));
        let range = start.min(range.end)..range.end;
        let vertices = &mut self.vertices;
        if let Some(xy) = vertices.xy.get_mut(range.clone()) {
            xy.reverse();
        }
        for column in [&mut vertices.z, &mut vertices.m].into_iter().flatten() {
            if let Some(values) = column.get_mut(range.clone()) {
                values.reverse();
            }
        }
        if let Some(id) = vertices.id.as_mut().and_then(|id| id.get_mut(range)) {
            id.reverse();
        }
    }
}

/// A polygon over these rings with the odd-even fill rule, every ring closed as the engine
/// keeps them, and its OGC flags computed.
impl From<MultiPath> for Polygon {
    fn from(rings: MultiPath) -> Self {
        let mut polygon = Polygon {
            rings,
            fill_rule: FillRule::OddEven,
        };
        polygon.update_ogc_flags();
        for flags in &mut polygon.rings.path_flags {
            flags.insert(PathFlag::Closed);
        }
        polygon
    }
}

impl Polygon {
    /// Flags each ring whose area has the sign of the first non-zero ring as an OGC polygon
    /// start, as the engine does for simple polygons (`_updateOGCFlagsHelper`).
    pub fn update_ogc_flags(&mut self) {
        let count = self.rings.path_count();
        self.rings.path_flags.resize(count, PathFlags::default());
        let mut first_sign = 0.0;
        for i in 0..count {
            let area = self.rings.ring_area(i).unwrap_or(0.0);
            if first_sign == 0.0 && area != 0.0 {
                first_sign = area.signum();
            }
            let flags = &mut self.rings.path_flags[i];
            if first_sign == 0.0 || area * first_sign > 0.0 {
                flags.insert(PathFlag::OgcStartPolygon);
            } else {
                flags.remove(PathFlag::OgcStartPolygon);
            }
        }
    }

    /// Ring ranges of the OGC polygons: a flagged ring followed by the holes up to the next one.
    /// The first ring always starts a polygon.
    pub fn ogc_polygons(&self) -> impl Iterator<Item = Range<usize>> + '_ {
        let count = self.rings.path_count();
        let mut starts = (0..count)
            .filter(|&i| {
                i == 0
                    || self
                        .rings
                        .path_flags
                        .get(i)
                        .is_some_and(|f| f.has(PathFlag::OgcStartPolygon))
            })
            .peekable();
        std::iter::from_fn(move || {
            let start = starts.next()?;
            Some(start..starts.peek().copied().unwrap_or(count))
        })
    }
}

#[cfg(test)]
mod tests;
