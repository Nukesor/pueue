use crossbeam_channel::Sender;

use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use super::SENDER_ERR;
use crate::network::response_helper::*;

/// Invoked when calling `pueue pause`.
/// Forward the pause message to the task handler, which then pauses groups/tasks/everything.
pub fn pause(message: PauseMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let state = state.lock().unwrap();
    if let Err(message) = ensure_group_exists(&state, &message.group) {
        return message;
    }

    sender
        .send(Message::Pause(message.clone()))
        .expect(SENDER_ERR);

    if !message.task_ids.is_empty() {
        let response = task_response_helper(
            "Tasks are being paused",
            message.task_ids,
            |task| matches!(task.status, TaskStatus::Running),
            &state,
        );
        return create_success_message(response);
    }
    if message.all {
        create_success_message("All queues are being paused.")
    } else {
        create_success_message(format!("Group \"{}\" is being paused.", &message.group))
    }
}
