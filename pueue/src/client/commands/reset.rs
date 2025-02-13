use pueue_lib::{client::Client, network::message::*};

use super::{handle_response, handle_user_confirmation};
use crate::{
    client::{commands::get_state, style::OutputStyle},
    internal_prelude::*,
};

/// Reset specific groups or the whole daemon.
///
/// The force flag determines whether the user confirmation should be skipped.
pub async fn reset(
    client: &mut Client,
    style: &OutputStyle,
    force: bool,
    groups: Vec<String>,
) -> Result<()> {
    // Get the current state and check if there're any running tasks.
    // If there are, ask the user if they really want to reset the state.
    let state = get_state(client).await?;

    // Get the groups that should be reset.
    let groups: Vec<String> = if groups.is_empty() {
        state.groups.keys().cloned().collect()
    } else {
        groups
    };

    // Check if there're any running tasks for that group
    let running_tasks = state
        .tasks
        .iter()
        .filter(|(_id, task)| groups.contains(&task.group))
        .filter_map(|(id, task)| if task.is_running() { Some(*id) } else { None })
        .collect::<Vec<_>>();

    if !running_tasks.is_empty() && !force {
        handle_user_confirmation("remove running tasks", &running_tasks)?;
    }

    let target = if groups.is_empty() {
        ResetTarget::All
    } else {
        ResetTarget::Groups(groups.clone())
    };

    client.send_request(ResetMessage { target }).await?;
    let response = client.receive_response().await?;

    handle_response(style, response)
}
