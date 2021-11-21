use std::sync::atomic::{AtomicUsize, Ordering};

/// The maximum logging level, stored statically in the memory as an atomic usize.
static MAX_LOG_LEVEL: AtomicUsize = AtomicUsize::new(0);

/// Set the maximum logging level. Accepts a usize.
///
/// The loggin levels use the following nomenclature:
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
        if $crate::log::max_level() >= 2 {
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

/// Formats an anyhow error chain in the following way:
///
/// ```ignore
/// - Chainlink 1 string
///   New line
/// - Chainlink 2 string
/// ```
pub fn format_chain(chain: anyhow::Chain) -> String {
    let mut out = String::new();

    fn begin_line_with(i: usize) -> String {
        if i == 0 { "- " } else { "  " }.to_owned()
    }

    if chain.len() > 1 {
        out.push_str("\nCaused by:\n");
        for link in chain.skip(1) {
            for (i, line) in link.to_string().lines().enumerate() {
                out.push('\n');
                out.push_str(&begin_line_with(i));
                out.push_str(line);
            }
        }
    }

    out
}
