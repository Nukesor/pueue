pub mod colors;
mod follow;
mod group;
pub mod helper;
mod log;
mod state;

use self::{colors::Colors, helper::style_text};

// Re-exports
pub use self::follow::follow_local_task_logs;
pub use self::group::print_groups;
pub use self::log::{determine_log_line_amount, print_logs};
pub use self::state::print_state;

/// Used to style any generic success message from the daemon.
pub fn print_success(_colors: &Colors, message: &str) {
    println!("{}", message);
}

/// Used to style any generic failure message from the daemon.
pub fn print_error(colors: &Colors, message: &str) {
    let styled = style_text(message, Some(colors.red()), None);
    println!("{}", styled);
}
