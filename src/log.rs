#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        let text = format!($($arg)*);
        let mut lines = text.lines();
        if let Some(line) = lines.next() {
            eprintln!("  \x1B[31mE\x1B[0m {}", line);
            for line in lines {
                eprintln!("  \x1B[90m│\x1B[0m {}", line);
            }
        }
    })
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        let text = format!($($arg)*);
        let mut lines = text.lines();
        if let Some(line) = lines.next() {
            eprintln!("  \x1B[33mW\x1B[0m {}", line);
            for line in lines {
                eprintln!("  \x1B[90m│\x1B[0m {}", line);
            }
        }
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        let text = format!($($arg)*);
        let mut lines = text.lines();
        if let Some(line) = lines.next() {
            println!("  \x1B[90m│ {}\x1B[0m", line);
            for line in lines {
                println!("  \x1B[90m│ {}\x1B[0m", line);
            }
        }
    }) 
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => ({
        let text = format!($($arg)*);
        let mut lines = text.lines();
        if let Some(line) = lines.next() {
            println!("  \x1B[32m✓\x1B[0m {}", line);
            for line in lines {
                println!("  \x1B[90m│ {}\x1B[0m", line);
            }
        }
    }) 
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        let text = format!($($arg)*);
        let mut lines = text.lines();
        if let Some(line) = lines.next() {
            println!("  D {}", line);
            for line in lines {
                println!("  \x1B[90m│\x1B[0m {}", line);
            }
        }
    })
}
