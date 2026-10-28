use serde_esri::enginex::geometry::Envelope2D;
use std::ops::RangeInclusive;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct XzRanges(pub Vec<RangeInclusive<i64>>);

impl XzRanges {
    pub fn new(full_extent: &Envelope2D, extents: &[Envelope2D], max_level: u8) -> Self {
        let max_level = i64::from(max_level);
        let mut ranges = Vec::new();
        for extent in extents {
            let x = ((full_extent.xmax - full_extent.xmin) / (extent.xmax - extent.xmin)).log2();
            let y = ((full_extent.ymax - full_extent.ymin) / (extent.ymax - extent.ymin)).log2();
            let query_level = (x.min(y).floor() + 1.0).min(max_level as f64) as i64;
            let depth_limit = query_level.saturating_add(4).min(max_level);
            let mut stack = vec![(*full_extent, 0_i64, 0_i64)];
            while let Some((cell, code, depth)) = stack.pop() {
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
                    let end = (depth..max_level)
                        .fold(code, |end, d| end + code_for_level(3, d, max_level));
                    ranges.push(code..=end);
                    continue;
                }
                ranges.push(code..=code);
                if depth == max_level {
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
                        code + code_for_level(quadrant, depth, max_level),
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

    pub fn contains(&self, code: i64) -> bool {
        self.first_ending_at_or_after(code)
            .is_some_and(|range| *range.start() <= code)
    }

    pub fn overlaps(&self, min: i64, max: i64) -> bool {
        self.first_ending_at_or_after(min)
            .is_some_and(|range| *range.start() <= max)
    }

    fn first_ending_at_or_after(&self, code: i64) -> Option<&RangeInclusive<i64>> {
        let index = self.0.partition_point(|range| *range.end() < code);
        self.0.get(index)
    }
}

fn code_for_level(quadrant: i64, depth: i64, max_level: i64) -> i64 {
    let levels = u32::try_from(max_level - depth).unwrap_or(0);
    quadrant * ((4_i64.saturating_pow(levels) - 1) / 3) + 1
}

#[cfg(test)]
mod tests;
