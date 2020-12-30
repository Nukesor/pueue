use std::sync::mpsc::Sender;

use pueue::network::message::*;
use pueue::state::SharedState;
use pueue::task::TaskStatus;

use super::SENDER_ERR;
use crate::network::response_helper::*;

/// Invoked when calling `pueue start`.
/// Forward the start message to the task handler, which then starts the process(es).
pub fn start(message: StartMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let state = state.lock().unwrap();
    if let Err(message) = ensure_group_exists(&state, &message.group) {
        return message;
    }

    sender
        .send(Message::Start(message.clone()))
        .expect(SENDER_ERR);

    if !message.task_ids.is_empty() {
        let response = task_response_helper(
            "Tasks are being started",
            message.task_ids,
            vec![TaskStatus::Paused, TaskStatus::Queued, TaskStatus::Stashed],
            &state,
        );
        return create_success_message(response);
    }

    if message.all {
        create_success_message("All queues are being resumed.")
    } else {
        create_success_message(format!("Group \"{}\" is being resumed.", &message.group))
    }
}
