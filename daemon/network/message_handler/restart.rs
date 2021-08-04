use crossbeam_channel::Sender;
use std::sync::MutexGuard;

use pueue_lib::aliasing::insert_alias;
use pueue_lib::network::message::*;
use pueue_lib::state::{SharedState, State};
use pueue_lib::task::TaskStatus;

use super::{task_action_response_helper, SENDER_ERR};

/// This is a small wrapper around the actual in-place task `restart` functionality.
///
/// The "not in-place" restart functionality is actually just a copy the finished task + create a
/// new task, which is completely handled on the client-side.
pub fn restart_multiple(
    message: RestartMessage,
    sender: &Sender<Message>,
    state: &SharedState,
) -> Message {
    let task_ids: Vec<usize> = message.tasks.iter().map(|task| task.task_id).collect();
    let mut state = state.lock().unwrap();

    // We have to compile the response beforehand.
    // Otherwise we no longer know which tasks, were actually capable of being being restarted.
    let response = task_action_response_helper(
        "Tasks restarted",
        task_ids.clone(),
        |task| task.is_done(),
        &state,
    );

    // Actually restart all tasks
    for task in message.tasks.iter() {
        restart(&mut state, task, message.stashed);
    }

    // Tell the task manager to start the task immediately if requested.
    if message.start_immediately {
        sender
            .send(Message::Start(StartMessage {
                tasks: TaskSelection::TaskIds(task_ids),
                children: false,
            }))
            .expect(SENDER_ERR);
    }

    response
}

/// This is invoked, whenever a task is actually restarted (in-place) without creating a new task.
/// Update a possibly changed path/command and reset all infos from the previous run.
///
/// The "not in-place" restart functionality is actually just a copy the finished task + create a
/// new task, which is completely handled on the client-side.
fn restart(state: &mut MutexGuard<State>, to_restart: &TasksToRestart, stashed: bool) {
    // Check if we actually know this task.
    let task = if let Some(task) = state.tasks.get_mut(&to_restart.task_id) {
        task
    } else {
        return;
    };

    // We cannot restart tasks that haven't finished yet.
    if !task.is_done() {
        return;
    }

    // Either enqueue the task or stash it.
    task.status = if stashed {
        TaskStatus::Stashed { enqueue_at: None }
    } else {
        TaskStatus::Queued
    };

    // Update command and path.
    task.original_command = to_restart.command.clone();
    task.command = insert_alias(to_restart.command.clone());
    task.path = to_restart.path.clone();

    // Reset all variables of any previous run.
    task.start = None;
    task.end = None;
}
