use pueue_lib::state::SharedState;
use pueue_lib::{network::message::*, settings::Settings};

use crate::daemon::network::response_helper::{ensure_group_exists, task_action_response_helper};
use crate::daemon::process_handler;

/// Invoked when calling `pueue kill`.
/// Forward the kill message to the task handler, which then kills the process.
pub fn kill(settings: &Settings, state: &SharedState, message: KillMessage) -> Message {
    let mut state = state.lock().unwrap();

    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(message) = ensure_group_exists(&mut state, group) {
            return message;
        }
    }

    // Construct a response depending on the selected tasks.
    let response = if let Some(signal) = &message.signal {
        match &message.tasks {
            TaskSelection::TaskIds(task_ids) => task_action_response_helper(
                "Tasks are being killed",
                task_ids.clone(),
                |task| task.is_running(),
                &state,
            ),
            TaskSelection::Group(group) => create_success_message(format!(
                "Sending signal {signal} to all running tasks of group {group}.",
            )),
            TaskSelection::All => {
                create_success_message(format!("Sending signal {signal} to all running tasks."))
            }
        }
    } else {
        match &message.tasks {
            TaskSelection::TaskIds(task_ids) => task_action_response_helper(
                "Tasks are being killed",
                task_ids.clone(),
                |task| task.is_running(),
                &state,
            ),
            TaskSelection::Group(group) => create_success_message(format!(
                "All tasks of group \"{group}\" are being killed. The group will also be paused!!!"
            )),
            TaskSelection::All => {
                create_success_message("All tasks are being killed. All groups will be paused!!!")
            }
        }
    };

    // Actually execute the command
    if let Message::Success(_) = response {
        process_handler::kill::kill(settings, &mut state, message.tasks, true, message.signal);
    }

    response
}
