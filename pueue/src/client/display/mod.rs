//! This module contains all logic for printing or displaying structured information about the
//! daemon.
//!
//! This includes formatting of task tables, group info, log inspection and log following.
mod follow;
mod group;
pub mod helper;
mod log;
mod state;
pub mod style;
pub mod table_builder;

use crossterm::style::Color;

// Re-exports
pub use self::follow::follow_local_task_logs;
pub use self::group::format_groups;
pub use self::log::{determine_log_line_amount, print_logs};
pub use self::state::print_state;
pub use self::style::OutputStyle;

/// Used to style any generic success message from the daemon.
pub fn print_success(_style: &OutputStyle, message: &str) {
    println!("{message}");
}

/// Used to style any generic failure message from the daemon.
pub fn print_error(style: &OutputStyle, message: &str) {
    let styled = style.style_text(message, Some(Color::Red), None);
    eprintln!("{styled}");
}
