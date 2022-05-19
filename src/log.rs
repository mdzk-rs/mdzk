use lazy_static::lazy_static;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The maximum logging level, stored statically in the memory as an atomic usize.
static MAX_LOG_LEVEL: AtomicUsize = AtomicUsize::new(0);

/// Set the maximum logging level. Accepts a usize.
///
/// The logging levels use the following nomenclature:
///
/// - 0: Off
/// - 1: Error
/// - 2: Warning and success
/// - 3: Info
/// - >4: Debug
#[inline]
pub fn set_max_level(level: usize) {
    MAX_LOG_LEVEL.store(level, Ordering::SeqCst);
}

#[inline(always)]
pub fn max_level() -> usize {
    MAX_LOG_LEVEL.load(Ordering::Relaxed)
}

lazy_static! {
    pub static ref ICONS: (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str
    ) = if std::env::var("NO_COLOR").is_ok() {
        ("E", "W", "✓", "·", "D")
    } else {
        (
            "\x1B[31mE\x1B[0m",
            "\x1B[33mW\x1B[0m",
            "\x1B[32m✓\x1B[0m",
            "·",
            "D",
        )
    };
}

pub fn log(icon: &str, text: String) {
    let mut lines = text.lines();
    if let Some(line) = lines.next() {
        eprintln!("{} {}", icon, line);
        for line in lines {
            eprintln!("  {}", line);
        }
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 1 {
            $crate::log::log($crate::log::ICONS.0, format!($($arg)*));
        }
    })
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 2 {
            $crate::log::log($crate::log::ICONS.1, format!($($arg)*));
        }
    })
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 2 {
            $crate::log::log($crate::log::ICONS.2, format!($($arg)*));
        }
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 3 {
            $crate::log::log($crate::log::ICONS.3, format!($($arg)*));
        }
    })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 4 {
            $crate::log::log($crate::log::ICONS.4, format!($($arg)*));
        }
    })
}
