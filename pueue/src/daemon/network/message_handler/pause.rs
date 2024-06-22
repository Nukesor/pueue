use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;
use pueue_lib::{network::message::*, settings::Settings};

use crate::daemon::network::response_helper::*;
use crate::daemon::process_handler;

/// Invoked when calling `pueue pause`.
/// Forward the pause message to the task handler, which then pauses groups/tasks/everything.
pub fn pause(settings: &Settings, state: &SharedState, message: PauseMessage) -> Message {
    let mut state = state.lock().unwrap();
    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(message) = ensure_group_exists(&mut state, group) {
            return message;
        }
    }

    // Construct a response depending on the selected tasks.
    let response = match &message.tasks {
        TaskSelection::TaskIds(task_ids) => task_action_response_helper(
            "Tasks are being paused",
            task_ids.clone(),
            |task| matches!(task.status, TaskStatus::Running),
            &state,
        ),
        TaskSelection::Group(group) => {
            create_success_message(format!("Group \"{group}\" is being paused."))
        }
        TaskSelection::All => create_success_message("All queues are being paused."),
    };

    // Actually execute the command
    if let Message::Success(_) = response {
        process_handler::pause::pause(settings, &mut state, message.tasks, message.wait);
    }

    response
}
