use comfy_table::Color;

mod follow;
mod group;
pub mod helper;
mod log;
mod state;

use self::helper::style_text;

// Re-exports
pub use self::follow::follow_local_task_logs;
pub use self::group::print_groups;
pub use self::log::print_logs;
pub use self::state::print_state;

/// Used to style any generic success message from the daemon.
pub fn print_success(message: &str) {
    println!("{}", message);
}

/// Used to style any generic failure message from the daemon.
pub fn print_error(message: &str) {
    let styled = style_text(message, Some(Color::Red), None);
    println!("{}", styled);
}
