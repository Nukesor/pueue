use pueue_lib::{Client, Settings, message::*, task::Task};

use super::{get_state, handle_response, handle_user_confirmation};
use crate::{client::style::OutputStyle, internal_prelude::*};

/// Tell the daemon to remove some tasks.
pub async fn remove(
    client: &mut Client,
    settings: Settings,
    style: &OutputStyle,
    task_ids: Vec<usize>,
    group: Option<String>,
    all: bool,
) -> Result<()> {
    // If --all or --group is specified, get all finished tasks
    let task_ids = if all || group.is_some() {
        let state = get_state(client).await?;
        let done_filter = |task: &Task| task.is_done();

        let filtered_tasks = if let Some(group_name) = group {
            state.filter_tasks_of_group(done_filter, &group_name)
        } else {
            state.filter_tasks(done_filter, None)
        };

        if filtered_tasks.matching_ids.is_empty() {
            println!("No finished tasks to remove.");
            return Ok(());
        }

        filtered_tasks.matching_ids
    } else if task_ids.is_empty() {
        bail!("Please provide the ids of the tasks you want to remove, use --all, or use --group.");
    } else {
        task_ids
    };

    if settings.client.show_confirmation_questions {
        handle_user_confirmation("remove", &task_ids)?;
    }
    client.send_request(Request::Remove(task_ids)).await?;

    let response = client.receive_response().await?;

    handle_response(style, response)
}
