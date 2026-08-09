use comfy_table::{presets, Table};
use console::style;

/// Create a styled table with the jm default settings.
pub fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table.load_style(presets::UTF8_FULL_CONDENSED);
    table.set_header(headers);
    table
}

/// Format a version string with highlighting.
pub fn format_version(version: &str, is_current: bool) -> String {
    if is_current {
        format!("{} {}", style(version).green().bold(), style("*").green())
    } else {
        version.to_string()
    }
}

/// Format an LTS badge.
pub fn format_lts(is_lts: bool) -> String {
    if is_lts {
        style("LTS").yellow().to_string()
    } else {
        String::new()
    }
}

/// Print an error message to stderr.
pub fn print_error(msg: &str) {
    eprintln!("{} {}", style("error:").red().bold(), msg);
}

/// Print a warning message to stderr.
pub fn print_warning(msg: &str) {
    eprintln!("{} {}", style("warning:").yellow().bold(), msg);
}

/// Print a success message.
pub fn print_success(msg: &str) {
    println!("{} {}", style("✓").green().bold(), msg);
}

/// Print an info message.
pub fn print_info(msg: &str) {
    println!("{} {}", style("→").cyan(), msg);
}
