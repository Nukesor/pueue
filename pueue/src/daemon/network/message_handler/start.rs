use pueue_lib::{network::message::*, settings::Settings, success_msg, task::TaskStatus};

use crate::daemon::{internal_state::SharedState, network::response_helper::*, process_handler};

/// Invoked when calling `pueue start`.
/// Forward the start message to the task handler, which then starts the process(es).
pub fn start(settings: &Settings, state: &SharedState, message: StartMessage) -> Response {
    let mut state = state.lock().unwrap();
    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(response) = ensure_group_exists(&mut state, group) {
            return response;
        }
    }

    let response = match &message.tasks {
        TaskSelection::TaskIds(task_ids) => task_action_response_helper(
            "Tasks have been started/resumed",
            task_ids.clone(),
            |task| {
                matches!(
                    task.status,
                    TaskStatus::Paused { .. }
                        | TaskStatus::Queued { .. }
                        | TaskStatus::Stashed { .. }
                )
            },
            &state,
        ),
        TaskSelection::Group(group) => {
            success_msg!("Group \"{group}\" is being resumed.")
        }
        TaskSelection::All => success_msg!("All groups are being resumed."),
    };

    if let Response::Success(_) = response {
        process_handler::start::start(settings, &mut state, message.tasks);
    }

    // Return a response depending on the selected tasks.
    response
}
