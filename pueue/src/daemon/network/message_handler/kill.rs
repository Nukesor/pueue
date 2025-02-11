use pueue_lib::{network::message::*, settings::Settings, success_msg, task::Task};

use crate::daemon::{
    internal_state::SharedState,
    network::response_helper::{ensure_group_exists, task_action_response_helper},
    process_handler,
};

/// Invoked when calling `pueue kill`.
/// Forward the kill message to the task handler, which then kills the process.
pub fn kill(settings: &Settings, state: &SharedState, message: KillMessage) -> Response {
    let mut state = state.lock().unwrap();

    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(response) = ensure_group_exists(&mut state, group) {
            return response;
        }
    }

    // Construct a response depending on the selected tasks.
    let response = if let Some(signal) = &message.signal {
        match &message.tasks {
            TaskSelection::TaskIds(task_ids) => task_action_response_helper(
                "Tasks are being killed",
                task_ids.clone(),
                Task::is_running,
                &state,
            ),
            TaskSelection::Group(group) => {
                success_msg!("Sending signal {signal} to all running tasks of group {group}.",)
            }
            TaskSelection::All => {
                success_msg!("Sending signal {signal} to all running tasks.")
            }
        }
    } else {
        match &message.tasks {
            TaskSelection::TaskIds(task_ids) => task_action_response_helper(
                "Tasks are being killed",
                task_ids.clone(),
                Task::is_running,
                &state,
            ),
            TaskSelection::Group(group) => success_msg!(
                "All tasks of group \"{group}\" are being killed. The group will also be paused!!!"
            ),
            TaskSelection::All => {
                success_msg!("All tasks are being killed. All groups will be paused!!!")
            }
        }
    };

    // Actually execute the command
    if let Response::Success(_) = response {
        process_handler::kill::kill(settings, &mut state, message.tasks, true, message.signal);
    }

    response
}
