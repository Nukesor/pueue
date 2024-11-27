use pueue_lib::aliasing::insert_alias;
use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;
use pueue_lib::{failure_msg, success_msg};

use super::*;
use crate::daemon::state_helper::save_state;
use crate::ok_or_save_state_failure;

/// Invoked when calling `pueue edit`.
/// If a user wants to edit a message, we need to send him the current command.
/// Lock the task to prevent execution, before the user has finished editing the command.
pub fn edit_request(state: &SharedState, task_ids: Vec<usize>) -> Message {
    // Check whether the task exists and is queued/stashed. Abort if that's not the case.
    let mut state = state.lock().unwrap();
    let mut editable_tasks: Vec<EditableTask> = Vec::new();
    for task_id in task_ids {
        match state.tasks.get_mut(&task_id) {
            Some(task) => {
                if !task.is_queued() && !task.is_stashed() {
                    return create_failure_message("You can only edit a queued/stashed task");
                }
                task.status = TaskStatus::Locked {
                    previous_status: Box::new(task.status.clone()),
                };

                editable_tasks.push(EditableTask::from(&*task));
            }
            None => return create_failure_message("No task with this id."),
        }
    }

    Message::EditResponse(editable_tasks)
}

/// Invoked after closing the editor on `pueue edit`.
/// Now we actually update the message with the updated command from the client.
pub fn edit(
    settings: &Settings,
    state: &SharedState,
    editable_tasks: Vec<EditableTask>,
) -> Message {
    // Check whether the task exists and is locked. Abort if that's not the case.
    let mut state = state.lock().unwrap();
    for editable_task in editable_tasks {
        match state.tasks.get_mut(&editable_task.id) {
            Some(task) => {
                let TaskStatus::Locked { previous_status } = &task.status else {
                    return create_failure_message(format!(
                        "Task {} is no longer locked.",
                        editable_task.id
                    ));
                };

                // Restore the task to its previous state.
                task.status = *previous_status.clone();

                // Update all properties to the edited values.
                task.original_command = editable_task.command.clone();
                task.command = insert_alias(settings, editable_task.command);
                task.path = editable_task.path;
                task.label = editable_task.label;
                task.priority = editable_task.priority;

                ok_or_save_state_failure!(save_state(&state, settings));
            }
            None => return failure_msg!("Task to edit has gone away: {}", editable_task.id),
        }
    }

    create_success_message("All tasks have been updated")
}

/// Invoked if a client fails to edit a task and asks the daemon to restore the task's status.
pub fn edit_restore(state: &SharedState, task_ids: Vec<usize>) -> Message {
    // Check whether the task exists and is queued/stashed. Abort if that's not the case.
    let mut state = state.lock().unwrap();
    let mut failed_tasks = Vec::new();
    for task_id in &task_ids {
        match state.tasks.get_mut(task_id) {
            Some(task) => {
                let TaskStatus::Locked { previous_status } = &task.status else {
                    failed_tasks.push(format!("Task {} isn't locked! Cannot be unlocked", task_id));
                    continue;
                };

                // Restore the task to its previous state.
                task.status = *previous_status.clone();
            }
            None => failed_tasks.push(format!("No task with id {}! Cannot be unlocked.", task_id)),
        }
    }

    // Return an error if any tasks couldn't be restored.
    if !failed_tasks.is_empty() {
        let mut error_msg = String::from("Some tasks couldn't be unlocked:\n");
        error_msg.push_str(&failed_tasks.join("\n"));
        return create_failure_message(error_msg);
    }

    success_msg!(
        "The requested task ids have been restored their previous state: {:?}",
        task_ids
    )
}
