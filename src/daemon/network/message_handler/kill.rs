use std::sync::mpsc::Sender;

use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use super::SENDER_ERR;
use crate::network::response_helper::task_response_helper;

/// Invoked when calling `pueue kill`.
/// Forward the kill message to the task handler, which then kills the process.
pub fn kill(message: KillMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    sender
        .send(Message::Kill(message.clone()))
        .expect(SENDER_ERR);

    if !message.task_ids.is_empty() {
        let state = state.lock().unwrap();
        let response = task_response_helper(
            "Tasks are being killed",
            message.task_ids,
            vec![TaskStatus::Running, TaskStatus::Paused],
            &state,
        );
        return create_success_message(response);
    }

    if message.all {
        create_success_message("All tasks are being killed.")
    } else {
        create_success_message(format!(
            "All tasks of group \"{}\" are being killed.",
            &message.group
        ))
    }
}
