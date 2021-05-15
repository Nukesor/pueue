use std::sync::mpsc::Sender;

use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::{Task, TaskStatus};

use super::*;
use crate::ok_or_return_failure_message;

/// Invoked when calling `pueue add`.
/// Queues a new task to the state.
/// If the start_immediately flag is set, send a StartMessage to the task handler.
pub fn add_task(message: AddMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    if let Err(message) = ensure_group_exists(&state, &message.group) {
        return message;
    }

    let starting_status = if message.stashed || message.enqueue_at.is_some() {
        TaskStatus::Stashed
    } else {
        TaskStatus::Queued
    };

    // Ensure that specified dependencies actually exist.
    let not_found: Vec<_> = message
        .dependencies
        .iter()
        .filter(|id| !state.tasks.contains_key(id))
        .collect();
    if !not_found.is_empty() {
        return create_failure_message(format!(
            "Unable to setup dependencies : task(s) {:?} not found",
            not_found
        ));
    }

    // Create a new task and add it to the state.
    let mut task = Task::new(
        message.command,
        message.path,
        message.envs,
        message.group,
        starting_status,
        message.enqueue_at,
        message.dependencies,
        message.label,
    );
    // Sort and deduplicate dependency id.
    task.dependencies.sort_unstable();
    task.dependencies.dedup();

    // Add a task.
    let task_id = state.add_task(task);

    // Notify the task handler, in case the client wants to start the task immediately.
    if message.start_immediately {
        sender
            .send(Message::Start(StartMessage {
                task_ids: vec![task_id],
                ..Default::default()
            }))
            .expect(SENDER_ERR);
    }
    // Create the customized response for the client.
    let message = if message.print_task_id {
        task_id.to_string()
    } else if let Some(enqueue_at) = message.enqueue_at {
        format!(
            "New task added (id {}). It will be enqueued at {}",
            task_id,
            enqueue_at.format("%Y-%m-%d %H:%M:%S")
        )
    } else {
        format!("New task added (id {}).", task_id)
    };

    // Add a task. This also persists the state.
    // Return an error, if this fails.
    ok_or_return_failure_message!(state.save());

    create_success_message(message)
}
