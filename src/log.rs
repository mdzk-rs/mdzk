use colored::{Colorize, ColoredString};

#[macro_export]
macro_rules! mdzk_error {
    ($($arg:tt)*) => {
        mdzk::msg::lay_out_text(&format!($($arg)*), "E".red())
    }
}

#[macro_export]
macro_rules! mdzk_warning {
    ($($arg:tt)*) => {
        mdzk::msg::lay_out_text(&format!($($arg)*), "W".yellow())
    }
}

pub fn lay_out_text(text: &str, icon: ColoredString) {
    let mut lines = text.lines();
    eprintln!("  {} {}", icon, lines.next().unwrap());
    for line in lines {
        eprintln!("  {} {}", "│".bright_black(), line);
    }
    eprintln!("  {}", "│".bright_black());
    // eprintln!("  {}", "╵".bright_black());
}
