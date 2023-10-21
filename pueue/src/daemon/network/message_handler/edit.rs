use pueue_lib::aliasing::insert_alias;
use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use super::*;
use crate::daemon::state_helper::save_state;
use crate::ok_or_return_failure_message;

/// Invoked when calling `pueue edit`.
/// If a user wants to edit a message, we need to send him the current command.
/// Lock the task to prevent execution, before the user has finished editing the command.
pub fn edit_request(task_id: usize, state: &SharedState) -> Message {
    // Check whether the task exists and is queued/stashed. Abort if that's not the case.
    let mut state = state.lock().unwrap();
    match state.tasks.get_mut(&task_id) {
        Some(task) => {
            if !task.is_queued() && !task.is_stashed() {
                return create_failure_message("You can only edit a queued/stashed task");
            }
            task.prev_status = task.status.clone();
            task.status = TaskStatus::Locked;

            EditResponseMessage {
                task_id: task.id,
                command: task.original_command.clone(),
                path: task.path.clone(),
                label: task.label.clone(),
            }
            .into()
        }
        None => create_failure_message("No task with this id."),
    }
}

/// Invoked after closing the editor on `pueue edit`.
/// Now we actually update the message with the updated command from the client.
pub fn edit(message: EditMessage, state: &SharedState, settings: &Settings) -> Message {
    // Check whether the task exists and is locked. Abort if that's not the case.
    let mut state = state.lock().unwrap();
    match state.tasks.get_mut(&message.task_id) {
        Some(task) => {
            if !(task.status == TaskStatus::Locked) {
                return create_failure_message("Task is no longer locked.");
            }

            // Restore the task to its previous state.
            task.status = task.prev_status.clone();

            // Update command if applicable.
            if let Some(command) = message.command {
                task.original_command = command.clone();
                task.command = insert_alias(settings, command);
            }
            // Update path if applicable.
            if let Some(path) = message.path {
                task.path = path;
            }
            // Update label if applicable.
            if message.label.is_some() {
                task.label = message.label;
            } else if message.delete_label {
                task.label = None;
            }

            ok_or_return_failure_message!(save_state(&state, settings));

            create_success_message("Command has been updated")
        }
        None => create_failure_message(format!("Task to edit has gone away: {}", message.task_id)),
    }
}

/// Invoked if a client fails to edit a task and asks the daemon to restore the task's status.
pub fn edit_restore(task_id: usize, state: &SharedState) -> Message {
    // Check whether the task exists and is queued/stashed. Abort if that's not the case.
    let mut state = state.lock().unwrap();
    match state.tasks.get_mut(&task_id) {
        Some(task) => {
            if task.status != TaskStatus::Locked {
                return create_failure_message("The requested task isn't locked");
            }
            task.status = task.prev_status.clone();

            create_success_message(format!(
                "The requested task's status has been restored to '{}'",
                task.status
            ))
        }
        None => create_failure_message("No task with this id."),
    }
}
