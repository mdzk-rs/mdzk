use anyhow::Error;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The maximum logging level, stored statically in the memory as an atomic usize.
static MAX_LOG_LEVEL: AtomicUsize = AtomicUsize::new(0);

/// Set the maximum logging level. Accepts a usize.
///
/// The loggin levels use the following nomenclature:
///
/// - 0: Off
/// - 1: Error and success
/// - 2: Warning
/// - 3: Info and success
/// - >4: Debug
#[inline]
pub fn set_max_level(level: usize) {
    MAX_LOG_LEVEL.store(level, Ordering::SeqCst);
}

#[inline(always)]
pub fn max_level() -> usize {
    MAX_LOG_LEVEL.load(Ordering::Relaxed)
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 1 {
            let text = format!($($arg)*);
            let mut lines = text.lines();
            if let Some(line) = lines.next() {
                eprintln!("  \x1B[31mE\x1B[0m {}", line);
                for line in lines {
                    eprintln!("  \x1B[90m│\x1B[0m {}", line);
                }
            }
        }
    })
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 1 {
            let text = format!($($arg)*);
            let mut lines = text.lines();
            if let Some(line) = lines.next() {
                println!("  \x1B[32m✓\x1B[0m {}", line);
                for line in lines {
                    println!("  \x1B[90m│\x1B[0m {}", line);
                }
            }
        }
    })
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 2 {
            let text = format!($($arg)*);
            let mut lines = text.lines();
            if let Some(line) = lines.next() {
                eprintln!("  \x1B[33mW\x1B[0m {}", line);
                for line in lines {
                    eprintln!("  \x1B[90m│\x1B[0m {}", line);
                }
            }
        }
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 3 {
            let text = format!($($arg)*);
            let mut lines = text.lines();
            if let Some(line) = lines.next() {
                println!("  \x1B[90m│ {}\x1B[0m", line);
                for line in lines {
                    println!("  \x1B[90m│ {}\x1B[0m", line);
                }
            }
        }
    })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        if $crate::log::max_level() >= 4 {
            let text = format!($($arg)*);
            let mut lines = text.lines();
            if let Some(line) = lines.next() {
                println!("  D {}", line);
                for line in lines {
                    println!("  \x1B[90m│\x1B[0m {}", line);
                }
            }
        }
    })
}

pub fn handle_anyhow_error(e: Error) {
    fn begin_line_with(i: usize) -> String {
        if i == 0 {
            "- "
        } else {
            "  "
        }.to_owned()
    }

    if e.chain().len() == 1 {
        error!("{}", e);
    } else {
        let rest = e
            .chain()
            .skip(1)
            .map(|chain| {
                chain
                    .to_string()
                    .lines()
                    .enumerate()
                    .map(|(i, line)| begin_line_with(i) + line)
                    .collect::<Vec<String>>()
                    .join("\n")
            })
            .collect::<Vec<String>>()
            .join("\n");

        error!("{}\n\n\x1B[4mCaused by:\x1B[0m\n\n{}", e, rest);
    }
}
