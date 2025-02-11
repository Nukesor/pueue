//! This module contains small helper functions that're useful for printing or displaying structured
//! information. Most of these functions are shared between multiple commands.
use crossterm::style::{Attribute, Color};
use pueue_lib::state::{Group, GroupStatus};

use super::style::OutputStyle;

/// Used to style any generic success message from the daemon.
pub fn print_success(_style: &OutputStyle, message: &str) {
    println!("{message}");
}

/// Used to style any generic failure message from the daemon.
pub fn print_error(style: &OutputStyle, message: &str) {
    let styled = style.style_text(message, Some(Color::Red), None);
    eprintln!("{styled}");
}

/// Return some nicely formatted info about a given group.
/// This is also used as a headline that's displayed above group's task tables.
pub fn get_group_headline(name: &str, group: &Group, style: &OutputStyle) -> String {
    // Style group name
    let name = style.style_text(format!("Group \"{name}\""), None, Some(Attribute::Bold));

    // Print the current state of the group.
    let status = match group.status {
        GroupStatus::Running => style.style_text("running", Some(Color::Green), None),
        GroupStatus::Paused => style.style_text("paused", Some(Color::Yellow), None),
        GroupStatus::Reset => style.style_text("resetting", Some(Color::Red), None),
    };

    format!("{} ({} parallel): {}", name, group.parallel_tasks, status)
}
