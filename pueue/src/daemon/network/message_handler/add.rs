use chrono::Local;
use pueue_lib::aliasing::insert_alias;
use pueue_lib::failure_msg;
use pueue_lib::network::message::*;
use pueue_lib::state::{GroupStatus, SharedState};
use pueue_lib::task::{Task, TaskStatus};

use super::*;
use crate::daemon::process_handler;
use crate::daemon::state_helper::save_state;
use crate::ok_or_save_state_failure;

/// Invoked when calling `pueue add`.
/// Queues a new task to the state.
/// If the start_immediately flag is set, send a StartMessage to the task handler.
pub fn add_task(settings: &Settings, state: &SharedState, message: AddMessage) -> Message {
    let mut state = state.lock().unwrap();
    if let Err(message) = ensure_group_exists(&mut state, &message.group) {
        return message;
    }

    // Ensure that specified dependencies actually exist.
    let not_found: Vec<_> = message
        .dependencies
        .iter()
        .filter(|id| !state.tasks.contains_key(id))
        .collect();
    if !not_found.is_empty() {
        return failure_msg!("Unable to setup dependencies : task(s) {not_found:?} not found",);
    }

    // Create a new task and add it to the state.
    let mut task = Task::new(
        message.command,
        message.path,
        message.envs,
        message.group,
        TaskStatus::Locked,
        message.dependencies,
        message.priority.unwrap_or(0),
        message.label,
    );

    // Set the starting status.
    if message.stashed || message.enqueue_at.is_some() {
        task.status = TaskStatus::Stashed {
            enqueue_at: message.enqueue_at,
        };
    } else {
        task.status = TaskStatus::Queued;
        task.enqueued_at = Some(Local::now());
    }

    // Check if there're any aliases that should be applied.
    // If one is found, we expand the command, otherwise we just take the original command.
    // Anyhow, we save this separately and keep the original command in a separate field.
    //
    // This allows us to have a debug experience and the user can opt to either show the
    // original command or the expanded command in their `status` view.
    task.command = insert_alias(settings, task.original_command.clone());

    // Sort and deduplicate dependency ids.
    task.dependencies.sort_unstable();
    task.dependencies.dedup();

    // Check if the task's group is paused before we pass it to the state
    let group_status = state
        .groups
        .get(&task.group)
        .expect("We ensured that the group exists.")
        .status;
    let group_is_paused = matches!(group_status, GroupStatus::Paused);

    // Add the task and persist the state.
    let task_id = state.add_task(task);
    ok_or_save_state_failure!(save_state(&state, settings));

    // Notify the task handler, in case the client wants to start the task immediately.
    if message.start_immediately {
        process_handler::start::start(settings, &mut state, TaskSelection::TaskIds(vec![task_id]));
    }

    // Create the customized response for the client.
    let mut response = if message.print_task_id {
        task_id.to_string()
    } else if let Some(enqueue_at) = message.enqueue_at {
        let enqueue_at = enqueue_at.format("%Y-%m-%d %H:%M:%S");
        format!("New task added (id {task_id}). It will be enqueued at {enqueue_at}")
    } else {
        format!("New task added (id {task_id}).")
    };

    // Notify the user if the task's group is paused
    if !message.print_task_id && group_is_paused {
        response.push_str("\nThe group of this task is currently paused!")
    }

    create_success_message(response)
}
