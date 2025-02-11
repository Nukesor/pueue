use crossterm::style::{Attribute, Color};

use pueue_lib::{
    network::message::response::GroupResponseMessage,
    state::{Group, GroupStatus},
};

use crate::client::cli::SubCommand;

use super::OutputStyle;

/// Print some info about the daemon's current groups.
/// This is used when calling `pueue group`.
pub fn format_groups(
    message: GroupResponseMessage,
    cli_command: &SubCommand,
    style: &OutputStyle,
) -> String {
    // Get commandline options to check whether we should return the groups as json.
    let json = match cli_command {
        SubCommand::Group { json, .. } => *json,
        // If `parallel` is called without an argument, the group info is shown.
        SubCommand::Parallel {
            parallel_tasks: None,
            group: None,
        } => false,
        _ => {
            panic!("Got wrong Subcommand {cli_command:?} in format_groups. This shouldn't happen.")
        }
    };

    if json {
        return serde_json::to_string(&message.groups).unwrap();
    }

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
    let name = style.style_text(format!("Group \"{name}\""), None, Some(Attribute::Bold));

    // Print the current state of the group.
    let status = match group.status {
        GroupStatus::Running => style.style_text("running", Some(Color::Green), None),
        GroupStatus::Paused => style.style_text("paused", Some(Color::Yellow), None),
        GroupStatus::Reset => style.style_text("resetting", Some(Color::Red), None),
    };

    format!("{} ({} parallel): {}", name, group.parallel_tasks, status)
}
