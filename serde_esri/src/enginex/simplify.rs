//! Douglas-Peucker simplification adapted from Esri's Apache-2.0 reference writer: <https://github.com/Esri/spatially-optimized-parquet/blob/ef5e577dd659fb62461f6f85739e3a0d076f29fa/crates/spatial/src/geometry/quantization.rs#L291-L416>

use crate::enginex::{geometry::MultiPath, vertex::VertexAttributes};

impl MultiPath {
    /// The original vertices Douglas-Peucker keeps within `tolerance`, measured in XY, or `None` when the multipath has curves.
    pub fn simplify(&self, tolerance: f64) -> Option<MultiPath> {
        fn select<T: Copy>(values: &[T], kept: &[usize]) -> Vec<T> {
            kept.iter()
                .filter_map(|&i| values.get(i).copied())
                .collect()
        }
        if self.has_nonlinear_segments() {
            return None;
        }
        let mut kept = Vec::with_capacity(self.vertices.len());
        let mut path_offsets = vec![0];
        for path in 0..self.path_count() {
            kept.extend(self.douglas_peucker(path, tolerance));
            path_offsets.push(i32::try_from(kept.len()).ok()?);
        }
        Some(MultiPath {
            vertices: VertexAttributes {
                xy: select(&self.vertices.xy, &kept),
                z: self.vertices.z.as_deref().map(|z| select(z, &kept)),
                m: self.vertices.m.as_deref().map(|m| select(m, &kept)),
                id: self.vertices.id.as_deref().map(|id| select(id, &kept)),
            },
            path_offsets,
            path_flags: self.path_flags.clone(),
            segments: None,
        })
    }

    fn douglas_peucker(&self, path: usize, tolerance: f64) -> Vec<usize> {
        let range = self.path_range(path).unwrap_or_default();
        let closing = (self.is_closed_path(path) && !range.is_empty()).then_some(range.start);
        let xy = self
            .vertices
            .xy
            .get(range.clone())
            .unwrap_or_default()
            .iter()
            .chain(closing.and_then(|i| self.vertices.xy.get(i)))
            .copied()
            .collect::<Vec<_>>();
        let farthest = |start: usize, end: usize| {
            let (&[ax, ay], &[bx, by]) = (xy.get(start)?, xy.get(end)?);
            let (dx, dy) = (bx - ax, by - ay);
            let length = dx * dx + dy * dy;
            xy.get(start + 1..end)?
                .iter()
                .zip(start + 1..)
                .map(|(&[x, y], index)| {
                    let along = if length == 0.0 {
                        0.0
                    } else {
                        (((x - ax) * dx + (y - ay) * dy) / length).clamp(0.0, 1.0)
                    };
                    let (ox, oy) = (x - ax - along * dx, y - ay - along * dy);
                    (index, ox * ox + oy * oy)
                })
                .fold(None::<(usize, f64)>, |farthest, (index, distance)| {
                    if farthest.is_none_or(|(_, max)| distance > max) {
                        Some((index, distance))
                    } else {
                        farthest
                    }
                })
        };
        let closed = xy.len() > 2 && xy.first() == xy.last();
        let positions = match xy.len() {
            count if count <= 2 || (closed && count <= 4) => (0..count).collect(),
            count => {
                let last = count - 1;
                let split = || {
                    let (far, _) = farthest(0, last)?;
                    let other = match (farthest(0, far), farthest(far, last)) {
                        (Some(before), Some(after)) if after.1 > before.1 => after.0,
                        (Some(before), _) => before.0,
                        (None, after) => after?.0,
                    };
                    Some((far.min(other), far.max(other)))
                };
                let mut sections = match closed.then(split).flatten() {
                    Some((first, second)) => vec![(second, last), (first, second), (0, first)],
                    None => vec![(0, last)],
                };
                let tolerance_squared = tolerance * tolerance;
                let mut kept = vec![0];
                while let Some((start, end)) = sections.pop() {
                    match farthest(start, end) {
                        Some((index, distance)) if distance > tolerance_squared => {
                            sections.push((index, end));
                            sections.push((start, index));
                        }
                        _ => kept.push(end),
                    }
                }
                kept
            }
        };
        positions
            .into_iter()
            .filter(|&position| position < range.len())
            .map(|position| range.start + position)
            .collect()
    }
}

#[cfg(test)]
mod tests;
