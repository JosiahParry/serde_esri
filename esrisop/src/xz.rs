use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_esri::enginex::geometry::Envelope2D;
use std::ops::RangeInclusive;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "u8", into = "u8")]
pub struct MaxLevel(u8);

impl TryFrom<u8> for MaxLevel {
    type Error = Error;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        if level > 31 {
            return Err(Error::MaxLevel(level));
        }
        Ok(MaxLevel(level))
    }
}

impl From<MaxLevel> for u8 {
    fn from(level: MaxLevel) -> Self {
        level.0
    }
}

impl MaxLevel {
    fn child_offset(self, quadrant: i64, depth: i64) -> i64 {
        let levels = u32::try_from(i64::from(self.0) - depth).unwrap_or(0);
        quadrant * ((4_i64.saturating_pow(levels) - 1) / 3) + 1
    }

    pub fn index_key(self, full_extent: &Envelope2D, extent: &Envelope2D) -> i64 {
        let width = full_extent.xmax - full_extent.xmin;
        let height = full_extent.ymax - full_extent.ymin;
        let (feature_width, feature_height) =
            (extent.xmax - extent.xmin, extent.ymax - extent.ymin);
        let mut level = if feature_width <= 0.0 || feature_height <= 0.0 {
            0
        } else {
            let x = (width / feature_width).log2();
            let y = (height / feature_height).log2();
            (x.min(y).floor() as u32 + 1).min(u32::from(self.0))
        };
        let cell_width = width / 2f64.powi(level as i32);
        let cell_height = height / 2f64.powi(level as i32);
        let cells = |min: f64, max: f64, origin: f64, size: f64| {
            ((max - origin) / size).floor() as i64 - ((min - origin) / size).floor() as i64 + 1
        };
        if cells(extent.xmin, extent.xmax, full_extent.xmin, cell_width) > 2
            || cells(extent.ymin, extent.ymax, full_extent.ymin, cell_height) > 2
        {
            level = level.saturating_sub(1);
        }
        let mut cell = *full_extent;
        let mut key = 0;
        for depth in 0..i64::from(level) {
            let xmid = (cell.xmin + cell.xmax) / 2.0;
            let ymid = (cell.ymin + cell.ymax) / 2.0;
            let right = extent.xmin >= xmid;
            let upper = extent.ymin >= ymid;
            if right {
                cell.xmin = xmid;
            } else {
                cell.xmax = xmid;
            }
            if upper {
                cell.ymin = ymid;
            } else {
                cell.ymax = ymid;
            }
            key += self.child_offset(i64::from(right) | i64::from(upper) << 1, depth);
        }
        key
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct XzRanges(pub Vec<RangeInclusive<i64>>);

impl XzRanges {
    pub fn new(full_extent: &Envelope2D, extents: &[Envelope2D], max_level: MaxLevel) -> Self {
        let deepest = i64::from(max_level.0);
        let mut ranges = Vec::new();
        for extent in extents {
            let x = ((full_extent.xmax - full_extent.xmin) / (extent.xmax - extent.xmin)).log2();
            let y = ((full_extent.ymax - full_extent.ymin) / (extent.ymax - extent.ymin)).log2();
            let query_level = (x.min(y).floor() + 1.0).min(deepest as f64) as i64;
            let depth_limit = query_level.saturating_add(4).min(deepest);
            let mut stack = vec![(*full_extent, 0_i64, 0_i64)];
            while let Some((cell, xz, depth)) = stack.pop() {
                let enlarged = Envelope2D {
                    xmax: cell.xmax + (cell.xmax - cell.xmin),
                    ymax: cell.ymax + (cell.ymax - cell.ymin),
                    ..cell
                };
                if enlarged.xmax <= extent.xmin
                    || enlarged.xmin >= extent.xmax
                    || enlarged.ymax <= extent.ymin
                    || enlarged.ymin >= extent.ymax
                {
                    continue;
                }
                let contained = enlarged.xmin >= extent.xmin
                    && enlarged.xmax <= extent.xmax
                    && enlarged.ymin >= extent.ymin
                    && enlarged.ymax <= extent.ymax;
                if depth == depth_limit || contained {
                    let end =
                        (depth..deepest).fold(xz, |end, d| end + max_level.child_offset(3, d));
                    ranges.push(xz..=end);
                    continue;
                }
                ranges.push(xz..=xz);
                if depth == deepest {
                    continue;
                }
                let xmid = (cell.xmin + cell.xmax) / 2.0;
                let ymid = (cell.ymin + cell.ymax) / 2.0;
                let quadrants = [
                    Envelope2D {
                        xmax: xmid,
                        ymax: ymid,
                        ..cell
                    },
                    Envelope2D {
                        xmin: xmid,
                        ymax: ymid,
                        ..cell
                    },
                    Envelope2D {
                        xmax: xmid,
                        ymin: ymid,
                        ..cell
                    },
                    Envelope2D {
                        xmin: xmid,
                        ymin: ymid,
                        ..cell
                    },
                ];
                for (quadrant, child) in (0_i64..).zip(quadrants) {
                    stack.push((
                        child,
                        xz + max_level.child_offset(quadrant, depth),
                        depth + 1,
                    ));
                }
            }
        }
        ranges.sort_by_key(|range| (*range.start(), *range.end()));
        let mut merged = Vec::<RangeInclusive<i64>>::with_capacity(ranges.len());
        for range in ranges {
            match merged.last_mut() {
                Some(last) if last.end().saturating_add(1) >= *range.start() => {
                    *last = *last.start()..=(*last.end()).max(*range.end());
                }
                _ => merged.push(range),
            }
        }
        XzRanges(merged)
    }

    pub fn overlaps(&self, min: i64, max: i64) -> bool {
        let index = self.0.partition_point(|range| *range.end() < min);
        self.0.get(index).is_some_and(|range| *range.start() <= max)
    }
}

#[cfg(test)]
mod tests;
