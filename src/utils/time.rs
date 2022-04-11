use std::sync::atomic::{AtomicI8, Ordering};
use time::{
    format_description::{well_known::Rfc3339, FormatItem},
    macros::{format_description, time},
    Date, OffsetDateTime, PrimitiveDateTime, UtcOffset,
};

const DATE: &[FormatItem] = format_description!("[year]-[month]-[day]");
const DATE_HM: &[FormatItem] = format_description!("[year]-[month]-[day] [hour]:[minute]");
const DATE_HMS: &[FormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
const READABLE: &[FormatItem] =
    format_description!("[day padding:none] [month repr:long case_sensitive:false] [year]");
const READABLE_AMERICAN: &[FormatItem] =
    format_description!("[month repr:long case_sensitive:false] [day padding:none] [year]");

// We store the current timezone's HMS values as atomic I8s. This allows for efficient loading and
// storing of this value, and makes it conveniently accessible in multi-threaded scenarios.
static TZ_H: AtomicI8 = AtomicI8::new(0);
static TZ_M: AtomicI8 = AtomicI8::new(0);
static TZ_S: AtomicI8 = AtomicI8::new(0);

#[inline(always)]
/// Finds the current timezone offset and stores it as atomic I8s.
///
/// Finding a local offset is only possible in single threaded scenarios. If this function is used
/// in a multi-thread scenario or if it otherwise fails, it will fallback to UTC (no offset.)
pub fn store_timezone() {
    if let Ok(offset) = UtcOffset::current_local_offset() {
        let (h, m, s) = offset.as_hms();
        TZ_H.store(h, Ordering::SeqCst);
        TZ_M.store(m, Ordering::SeqCst);
        TZ_S.store(s, Ordering::SeqCst);
    }
}

#[inline(always)]
/// Loads a [`time::UtcOffset`] from the currently stored timezone offset (see [`store_timezone`].)
pub fn load_timezone() -> UtcOffset {
    let h = TZ_H.load(Ordering::Relaxed);
    let m = TZ_M.load(Ordering::Relaxed);
    let s = TZ_S.load(Ordering::Relaxed);
    // Safe unwrap. The stored values cannot exceed the allowed range.
    UtcOffset::from_hms(h, m, s).unwrap()
}

/// Attempts to interpret the supplied string (or string-like) as a datetime or date.
///
/// The function tries to match on the following patterns in order:
///
/// - [RFC3339](https://datatracker.ietf.org/doc/html/rfc3339) with centiseconds: `1814-05-17T07:20:42.52+02:00`
/// - [RFC3339](https://datatracker.ietf.org/doc/html/rfc3339) without centiseconds: `1814-05-17T07:20:42+02:00`
/// - Naive datetimes: `1814-05-17 07:20:42`
/// - Naive datetimes without seconds: `1814-05-17 07:20`
/// - Pure dates (time is set to midnight): `1814-05-17`
/// - Readable dates (time is set to midnight): `17 May 1814`
/// - Readable dates on american form (time is set to midnight): `May 17 1814`
///
/// If a match is found, the function will short-circuit and return the corresponding
/// [`OffsetDateTime`]. If no match is found, the next pattern is tried. If no
/// pattern matches, the return value is [`None`].
///
/// When no timezone is specified, the local timezone of the computer will be used (if available -
/// the fallback is UTC.) This detail is worth being aware of when running mdzk via CI.
///
/// # Example
///
/// ```
/// use time::macros::datetime;
/// use mdzk::utils::time::parse_datestring;
///
/// let independence_day = datetime!(1776-07-04 0:00:00 -4);
/// assert_eq!(independence_day, parse_datestring("1776-07-04T00:00:00-04:00").unwrap());
/// ```
pub fn parse_datestring(datestring: impl AsRef<str>) -> Option<OffsetDateTime> {
    let datestring = datestring.as_ref();

    OffsetDateTime::parse(datestring, &Rfc3339)
        .or_else(|_| {
            PrimitiveDateTime::parse(datestring, DATE_HMS)
                .or_else(|_| PrimitiveDateTime::parse(datestring, DATE_HM))
                .or_else(|_| {
                    Date::parse(datestring, DATE)
                        .or_else(|_| Date::parse(datestring, READABLE))
                        .or_else(|_| Date::parse(datestring, READABLE_AMERICAN))
                        .map(|d| PrimitiveDateTime::new(d, time!(0:00)))
                })
                .map(|dt| dt.assume_offset(load_timezone()))
        })
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    #[test]
    fn test_parse_datestring() {
        assert_eq!(
            UtcOffset::UTC,
            parse_datestring("1814-05-17T07:20:42Z").unwrap().offset()
        );
        assert_eq!(
            (7, 20, 42),
            parse_datestring("1814-05-17 07:20:42").unwrap().to_hms()
        );
        assert_eq!(
            (7, 20, 0),
            parse_datestring("1814-05-17 07:20").unwrap().to_hms()
        );
        assert_eq!(
            (1814, Month::May, 17),
            parse_datestring("1814-05-17").unwrap().to_calendar_date()
        );
        assert_eq!(
            (1814, Month::May, 17),
            parse_datestring("17 May 1814").unwrap().to_calendar_date()
        );
        assert_eq!(
            (1857, Month::March, 8),
            parse_datestring("March 8 1857").unwrap().to_calendar_date()
        );
    }
}
