use pueue_lib::{client::Client, network::message::*};

use super::handle_response;
use crate::{
    client::{cli::GroupCommand, display_helper::get_group_headline, style::OutputStyle},
    internal_prelude::*,
};

/// Add, remove a group or simply list all groups.
pub async fn group(
    client: &mut Client,
    style: &OutputStyle,
    cmd: Option<GroupCommand>,
    json: bool,
) -> Result<()> {
    let request = match cmd {
        Some(GroupCommand::Add { name, parallel }) => GroupMessage::Add {
            name: name.to_owned(),
            parallel_tasks: parallel.to_owned(),
        },
        Some(GroupCommand::Remove { name }) => GroupMessage::Remove(name.to_owned()),
        None => GroupMessage::List,
    };

    client.send_request(request).await?;

    let response = client.receive_response().await?;

    if let Response::Group(groups) = response {
        let group_text = format_groups(groups, style, json);
        println!("{group_text}");
        return Ok(());
    }

    handle_response(style, response)
}

/// Print some info about the daemon's current groups.
/// This is used when calling `pueue group`.
pub fn format_groups(message: GroupResponseMessage, style: &OutputStyle, json: bool) -> String {
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
