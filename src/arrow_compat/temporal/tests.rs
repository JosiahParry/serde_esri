use super::*;

#[test]
fn dates() {
    assert_eq!("1970-01-01".parse(), Ok(DateOnly(0)));
    assert_eq!("2003-01-25".parse(), Ok(DateOnly(12_077)));
    assert_eq!("1969-12-31".parse(), Ok(DateOnly(-1)));
    assert_eq!("2024-02-29".parse(), Ok(DateOnly(19_782)));
    assert_eq!("2024-13-01".parse::<DateOnly>(), Err(()));
    assert_eq!("yesterday".parse::<DateOnly>(), Err(()));
}

#[test]
fn times() {
    assert_eq!("21:00:00".parse(), Ok(TimeOnly(75_600_000)));
    assert_eq!("00:00:01.5".parse(), Ok(TimeOnly(1_500)));
    assert_eq!("12:30:45.123456".parse(), Ok(TimeOnly(45_045_123)));
    assert_eq!("25:00:00".parse::<TimeOnly>(), Err(()));
}

/// The documented example, 2003-01-25T14:35:00.927 at -08:00, is 22:35:00.927 UTC.
#[test]
fn offset_timestamps() {
    let utc = 12_077 * MS_PER_DAY + ((22 * 60 + 35) * 60) * 1000 + 927;
    assert_eq!("2003-01-25T14:35:00.927-08:00".parse(), Ok(TimestampOffset(utc)));
    assert_eq!("2003-01-25T22:35:00.927Z".parse(), Ok(TimestampOffset(utc)));
    assert_eq!("2003-01-26 04:05:00.927+05:30".parse(), Ok(TimestampOffset(utc)));
    assert_eq!("2003-01-25".parse::<TimestampOffset>(), Err(()));
}
