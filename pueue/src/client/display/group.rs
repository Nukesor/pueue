use comfy_table::{Attribute, Color};

use pueue_lib::{
    network::message::GroupResponseMessage,
    state::{Group, GroupStatus},
};

use super::OutputStyle;

/// Print some info about the daemon's current groups.
/// This is used when calling `pueue group`.
pub fn format_groups(message: GroupResponseMessage, style: &OutputStyle) -> String {
    let mut text = String::new();
    let mut group_iter = message.groups.iter().peekable();
    while let Some((name, group)) = group_iter.next() {
        let styled = get_group_headline(name, group, style);

        text.push_str(&styled);
        if group_iter.peek().is_some() {
            text.push('\n');
        }
    }

    text
}

/// Return some nicely formatted info about a given group.
/// This is also used as a headline that's displayed above group's task tables.
pub fn get_group_headline(name: &str, group: &Group, style: &OutputStyle) -> String {
    // Style group name
    let name = style.style_text(format!("Group \"{}\"", name), None, Some(Attribute::Bold));

    // Print the current state of the group.
    let status = match group.status {
        GroupStatus::Running => style.style_text("running", Some(Color::Green), None),
        GroupStatus::Paused => style.style_text("paused", Some(Color::Yellow), None),
    };

    format!("{} ({} parallel): {}", name, group.parallel_tasks, status)
}
