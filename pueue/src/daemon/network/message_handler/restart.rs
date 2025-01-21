use chrono::Local;
use std::sync::MutexGuard;

use pueue_lib::{
    aliasing::insert_alias,
    network::message::*,
    settings::Settings,
    state::{SharedState, State},
    task::{Task, TaskStatus},
};

use crate::daemon::process_handler;

use super::task_action_response_helper;

/// This is a small wrapper around the actual in-place task `restart` functionality.
///
/// The "not in-place" restart functionality is actually just a copy the finished task + create a
/// new task, which is completely handled on the client-side.
pub fn restart_multiple(
    settings: &Settings,
    state: &SharedState,
    message: RestartMessage,
) -> Message {
    let task_ids: Vec<usize> = message.tasks.iter().map(|task| task.task_id).collect();
    let mut state = state.lock().unwrap();

    // We have to compile the response beforehand.
    // Otherwise we no longer know which tasks, were actually capable of being being restarted.
    let response = task_action_response_helper(
        "Tasks has restarted",
        task_ids.clone(),
        Task::is_done,
        &state,
    );

    // Restart a tasks in-place
    for task in message.tasks {
        restart(&mut state, task, message.stashed, settings);
    }

    // Actually start the processes if we should do so.
    if message.start_immediately {
        process_handler::start::start(settings, &mut state, TaskSelection::TaskIds(task_ids));
    }

    response
}

/// This is invoked, whenever a task is actually restarted (in-place) without creating a new task.
/// Update a possibly changed path/command/label and reset all infos from the previous run.
///
/// The "not in-place" restart functionality is actually just a copy the finished task + create a
/// new task, which is completely handled on the client-side.
fn restart(
    state: &mut MutexGuard<State>,
    to_restart: TaskToRestart,
    stashed: bool,
    settings: &Settings,
) {
    // Check if we actually know this task.
    let Some(task) = state.tasks.get_mut(&to_restart.task_id) else {
        return;
    };

    // We cannot restart tasks that haven't finished yet.
    if !task.is_done() {
        return;
    }

    // Either enqueue the task or stash it.
    if stashed {
        task.status = TaskStatus::Stashed { enqueue_at: None };
    } else {
        task.status = TaskStatus::Queued {
            enqueued_at: Local::now(),
        };
    };

    // Update task properties in case they've been edited.
    task.original_command = to_restart.command.clone();
    task.command = insert_alias(settings, to_restart.command);
    task.path = to_restart.path;
    task.label = to_restart.label.clone();
    task.priority = to_restart.priority;
}
