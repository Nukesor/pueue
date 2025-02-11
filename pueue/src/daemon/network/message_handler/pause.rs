use pueue_lib::{network::message::*, settings::Settings, success_msg, task::TaskStatus};

use crate::daemon::{internal_state::SharedState, network::response_helper::*, process_handler};

/// Invoked when calling `pueue pause`.
/// Forward the pause message to the task handler, which then pauses groups/tasks/everything.
pub fn pause(settings: &Settings, state: &SharedState, message: PauseMessage) -> Response {
    let mut state = state.lock().unwrap();
    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(response) = ensure_group_exists(&mut state, group) {
            return response;
        }
    }

    // Construct a response depending on the selected tasks.
    let response = match &message.tasks {
        TaskSelection::TaskIds(task_ids) => task_action_response_helper(
            "Tasks have been paused",
            task_ids.clone(),
            |task| matches!(task.status, TaskStatus::Running { .. }),
            &state,
        ),
        TaskSelection::Group(group) => {
            success_msg!("Group \"{group}\" is being paused.")
        }
        TaskSelection::All => success_msg!("All groups are being paused."),
    };

    // Actually execute the command
    if let Response::Success(_) = response {
        process_handler::pause::pause(settings, &mut state, message.tasks, message.wait);
    }

    response
}
