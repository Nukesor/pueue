use crossbeam_channel::Sender;
use std::sync::MutexGuard;

use pueue_lib::aliasing::insert_alias;
use pueue_lib::network::message::*;
use pueue_lib::state::{SharedState, State};
use pueue_lib::task::TaskStatus;

use super::SENDER_ERR;

/// This is a small wrapper around the actual in-place task `restart` functionality.
pub fn restart_multiple(
    message: RestartMessage,
    sender: &Sender<Message>,
    state: &SharedState,
) -> Message {
    let mut state = state.lock().unwrap();
    for task in message.tasks.iter() {
        restart(&mut state, task, message.stashed);
    }

    // Tell the task manager to start the task immediately, if it's requested.
    if message.start_immediately {
        let task_ids = message.tasks.iter().map(|task| task.task_id).collect();
        sender
            .send(Message::Start(StartMessage {
                tasks: TaskSelection::TaskIds(task_ids),
                children: false,
            }))
            .expect(SENDER_ERR);
    }

    create_success_message("Tasks restarted")
}

/// This is invoked, whenever a task is actually restarted (in-place) without creating a new task.
/// Update a possibly changed path/command and reset all infos from the previous run.
fn restart(state: &mut MutexGuard<State>, to_restart: &TasksToRestart, stashed: bool) {
    // Check if we actually know this task.
    let task = if let Some(task) = state.tasks.get_mut(&to_restart.task_id) {
        task
    } else {
        return;
    };

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
