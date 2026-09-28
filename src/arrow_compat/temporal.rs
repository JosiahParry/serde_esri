//! Parsers for the ISO 8601 strings services use for date-only, time-only, and offset timestamp
//! fields, as the Query operation documents them: `"2003-01-25"`, `"21:00:00"`, and
//! `"2003-01-25T14:35:00.927-08:00"`. Fractions of a second beyond milliseconds are dropped.

use std::str::FromStr;

const MS_PER_DAY: i64 = 86_400_000;

/// A date-only value as days since the Unix epoch, Arrow's `Date32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DateOnly(pub(super) i32);

/// A time-only value as milliseconds since midnight, Arrow's `Time32(Millisecond)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TimeOnly(pub(super) i32);

/// An offset timestamp as UTC milliseconds since the Unix epoch; the offset itself is dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TimestampOffset(pub(super) i64);

/// Parses `YYYY-MM-DD` into days since the epoch (Howard Hinnant's `days_from_civil`).
impl FromStr for DateOnly {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let mut parts = s.splitn(3, '-');
        let mut next = || parts.next().ok_or(());
        let year: i64 = next()?.parse().map_err(|_| ())?;
        let month: i64 = next()?.parse().map_err(|_| ())?;
        let day: i64 = next()?.parse().map_err(|_| ())?;
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return Err(());
        }
        let y = if month <= 2 { year - 1 } else { year };
        let era = y.div_euclid(400);
        let year_of_era = y - era * 400;
        let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        let days = era * 146_097 + day_of_era - 719_468;
        i32::try_from(days).map(DateOnly).map_err(|_| ())
    }
}

/// Parses `HH:MM:SS` with optional fractional seconds into milliseconds since midnight.
impl FromStr for TimeOnly {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let (clock, fraction) = s.split_once('.').unwrap_or((s, ""));
        let mut parts = clock.splitn(3, ':');
        let mut next = || -> Result<i32, ()> { parts.next().ok_or(())?.parse().map_err(|_| ()) };
        let (hours, minutes, seconds) = (next()?, next()?, next()?);
        if !(0..24).contains(&hours) || !(0..60).contains(&minutes) || !(0..61).contains(&seconds) {
            return Err(());
        }
        let millis = if fraction.is_empty() {
            0
        } else {
            let digits: String = fraction.chars().chain("000".chars()).take(3).collect();
            digits.parse().map_err(|_| ())?
        };
        Ok(TimeOnly(((hours * 60 + minutes) * 60 + seconds) * 1000 + millis))
    }
}

/// Parses `YYYY-MM-DDTHH:MM:SS[.fff](Z|±HH:MM)`, with a space accepted in place of `T` and a
/// missing offset read as UTC.
impl FromStr for TimestampOffset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let (date, rest) = s.split_once(['T', ' ']).ok_or(())?;
        let (clock, offset_minutes) = match rest.rfind(['+', '-', 'Z']) {
            Some(i) if rest[i..].starts_with('Z') => (&rest[..i], 0),
            Some(i) => {
                let sign = if rest[i..].starts_with('-') { -1 } else { 1 };
                let (h, m) = rest[i + 1..].split_once(':').unwrap_or((&rest[i + 1..], "0"));
                let h: i64 = h.parse().map_err(|_| ())?;
                let m: i64 = m.parse().map_err(|_| ())?;
                (&rest[..i], sign * (h * 60 + m))
            }
            None => (rest, 0),
        };
        let DateOnly(days) = date.parse()?;
        let TimeOnly(millis) = clock.parse()?;
        let local = i64::from(days) * MS_PER_DAY + i64::from(millis);
        Ok(TimestampOffset(local - offset_minutes * 60_000))
    }
}

#[cfg(test)]
mod tests;
