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

/// The current timezone's hour component
static TZ_H: AtomicI8 = AtomicI8::new(0);
/// The current timezone's minute component
static TZ_M: AtomicI8 = AtomicI8::new(0);
/// The current timezone's second component
static TZ_S: AtomicI8 = AtomicI8::new(0);

#[inline(always)]
/// Stores the current timezone offset as atomic I8s
///
/// Finding a local offset is only possible in single threaded scenarios. If this function is used
/// in a multi-thread scenario or if it otherwise fails, it will fallback to UTC (no offset.)
pub fn set_timezone() {
    if let Ok(offset) = UtcOffset::current_local_offset() {
        let (h, m, s) = offset.as_hms();
        TZ_H.store(h, Ordering::SeqCst);
        TZ_M.store(m, Ordering::SeqCst);
        TZ_S.store(s, Ordering::SeqCst);
    }
}

#[inline(always)]
/// Loads a [`time::UtcOffset`] from the current timezone offset stored by [`set_timezone`]
pub fn load_timezone() -> UtcOffset {
    let h = TZ_H.load(Ordering::Relaxed);
    let m = TZ_M.load(Ordering::Relaxed);
    let s = TZ_S.load(Ordering::Relaxed);
    // Safe unwrap. The values cannot exceed the range since they are UTC or loaded from current TZ
    UtcOffset::from_hms(h, m, s).unwrap()
}

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
