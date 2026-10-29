use super::*;

fn extent(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Envelope2D {
    Envelope2D {
        xmin,
        ymin,
        xmax,
        ymax,
    }
}

fn ranges(full: Envelope2D, query: Envelope2D, max_level: u8) -> Vec<RangeInclusive<i64>> {
    XzRanges::new(&full, &[query], MaxLevel(max_level)).0
}

#[test]
fn square_extent_cases() {
    let full = extent(0.0, 0.0, 10.0, 10.0);
    let cases = [
        (extent(0.0, 0.0, 10.0, 10.0), 0, vec![0..=0]),
        (extent(0.0, 0.0, 10.0, 10.0), 1, vec![0..=4]),
        (extent(0.0, 0.0, 4.9, 4.9), 1, vec![0..=1]),
        (extent(0.0, 0.0, 5.0, 4.9), 1, vec![0..=1]),
        (extent(0.0, 0.0, 5.000001, 4.9), 1, vec![0..=2]),
        (extent(0.0, 0.0, 4.9, 5.0), 1, vec![0..=1]),
        (extent(0.0, 0.0, 4.9, 5.000001), 1, vec![0..=1, 3..=3]),
        (
            extent(5.0, 5.0, 10.0, 10.0),
            2,
            vec![0..=1, 5..=6, 9..=11, 13..=13, 15..=20],
        ),
        (extent(4.9, 5.0, 10.0, 10.0), 2, vec![0..=1, 4..=6, 9..=20]),
        (extent(4.5, 0.0, 5.5, 0.25), 2, vec![0..=3, 6..=7]),
        (extent(0.0, 0.0, 5.0, 5.0), 3, vec![0..=21]),
        (extent(0.0, 0.0, 5.0, 5.0), 4, vec![0..=85]),
        (extent(0.0, 0.0, 5.0, 5.0), 8, vec![0..=21_845]),
        (extent(0.0, 0.0, 5.0, 5.0), 12, vec![0..=5_592_405]),
        (extent(0.0, 0.0, 5.0, 5.0), 20, vec![0..=366_503_875_925]),
        (extent(-2.0, -2.0, -1.0, -1.0), 2, vec![]),
    ];
    for (query, max_level, expected) in cases {
        assert_eq!(
            ranges(full, query, max_level),
            expected,
            "{query:?} at {max_level}"
        );
    }
}

#[test]
fn rectangular_extent_cases() {
    let cases = [
        (
            extent(0.0, 0.0, 20.0, 10.0),
            extent(9.0, 0.0, 11.0, 0.25),
            2,
            vec![0..=3, 6..=7],
        ),
        (
            extent(0.0, 0.0, 10.0, 20.0),
            extent(0.0, 9.0, 0.25, 11.0),
            2,
            vec![0..=2, 4..=4, 11..=12],
        ),
        (
            extent(10.25, 20.25, 30.25, 28.25),
            extent(20.25, 24.25, 30.25, 28.25),
            2,
            vec![0..=1, 5..=6, 9..=11, 13..=13, 15..=20],
        ),
        (
            extent(0.0, 0.0, 20.0, 10.0),
            extent(0.0, 0.0, 10.0, 1.0),
            4,
            vec![0..=12, 23..=33],
        ),
        (
            extent(0.0, 0.0, 20.0, 10.0),
            extent(0.0, 0.0, 2.0, 5.0),
            4,
            vec![0..=7, 13..=17, 44..=49, 55..=59],
        ),
        (
            extent(10.25, 20.25, 30.25, 28.25),
            extent(13.25, 21.25, 25.25, 22.25),
            3,
            vec![0..=11, 22..=27],
        ),
    ];
    for (full, query, max_level, expected) in cases {
        assert_eq!(
            ranges(full, query, max_level),
            expected,
            "{query:?} in {full:?}"
        );
    }
}

#[test]
fn extents_merge_into_one_set() {
    let full = extent(0.0, 0.0, 10.0, 10.0);
    let (lower, upper) = (extent(0.0, 0.0, 4.9, 4.9), extent(5.0, 5.0, 10.0, 10.0));
    let both = XzRanges::new(&full, &[lower, upper], MaxLevel(2));
    for xz in 0..=21 {
        let either = ranges(full, lower, 2)
            .iter()
            .chain(&ranges(full, upper, 2))
            .any(|range| range.contains(&xz));
        assert_eq!(both.overlaps(xz, xz), either, "xz {xz}");
    }
    assert!(both.0.windows(2).all(|w| w[0].end() + 1 < *w[1].start()));
}

#[test]
fn overlaps() {
    let ranges = XzRanges(vec![0..=1, 5..=6, 9..=11]);
    assert!(ranges.overlaps(0, 0));
    assert!(ranges.overlaps(6, 6));
    assert!(!ranges.overlaps(4, 4));
    assert!(!ranges.overlaps(12, 12));
    assert!(ranges.overlaps(2, 5));
    assert!(ranges.overlaps(11, 40));
    assert!(!ranges.overlaps(2, 4));
    assert!(!ranges.overlaps(12, 40));
    assert!(!XzRanges::default().overlaps(i64::MIN, i64::MAX));
}

#[test]
fn index_keys_match_the_reference_writer() {
    let full = extent(0.0, 0.0, 8.0, 8.0);
    let cases = [
        (extent(0.0, 0.0, 1.0, 1.0), 1, 1),
        (extent(7.0, 7.0, 8.0, 8.0), 1, 4),
        (extent(0.0, 0.0, 0.9, 0.9), 2, 2),
        (full, 3, 0),
        (extent(3.0, 4.2, 4.9, 4.9), 3, 51),
    ];
    for (feature, max_level, key) in cases {
        assert_eq!(MaxLevel(max_level).index_key(&full, &feature), key);
    }
    let full = extent(-117.3150315, 33.989629, -116.9566474, 34.1729119);
    let feature = extent(-117.0992242, 34.0448088, -117.085404, 34.0545866);
    assert_eq!(MaxLevel(20).index_key(&full, &feature), 555_482_436_952);
}

#[test]
fn index_keys_fall_inside_the_ranges_of_any_query_that_touches_them() {
    let full = extent(-180.0, -90.0, 180.0, 90.0);
    let features = [
        extent(-105.1, 39.6, -104.8, 39.8),
        extent(10.0, -5.0, 10.5, -4.5),
        extent(-179.9, -89.9, 179.9, 89.9),
        extent(0.001, 0.001, 0.002, 0.002),
    ];
    for feature in features {
        let key = MaxLevel(20).index_key(&full, &feature);
        let ranges = XzRanges::new(&full, &[feature], MaxLevel(20));
        assert!(ranges.overlaps(key, key), "{feature:?} key {key}");
    }
}
