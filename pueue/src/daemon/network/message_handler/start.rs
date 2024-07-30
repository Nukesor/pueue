use pueue_lib::network::message::*;
use pueue_lib::settings::Settings;
use pueue_lib::state::SharedState;
use pueue_lib::success_msg;
use pueue_lib::task::TaskStatus;

use crate::daemon::network::response_helper::*;
use crate::daemon::process_handler;

/// Invoked when calling `pueue start`.
/// Forward the start message to the task handler, which then starts the process(es).
pub fn start(settings: &Settings, state: &SharedState, message: StartMessage) -> Message {
    let mut state = state.lock().unwrap();
    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(message) = ensure_group_exists(&mut state, group) {
            return message;
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

    if let Message::Success(_) = response {
        process_handler::start::start(settings, &mut state, message.tasks);
    }

    // Return a response depending on the selected tasks.
    response
}
