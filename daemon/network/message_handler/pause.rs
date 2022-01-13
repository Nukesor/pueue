use crossbeam_channel::Sender;

use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use super::SENDER_ERR;
use crate::network::response_helper::*;

/// Invoked when calling `pueue pause`.
/// Forward the pause message to the task handler, which then pauses groups/tasks/everything.
pub fn pause(message: PauseMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(message) = ensure_group_exists(&mut state, group) {
            return message;
        }
    }

    // Forward the message to the task handler.
    sender
        .send(Message::Pause(message.clone()))
        .expect(SENDER_ERR);

    // Return a response depending on the selected tasks.
    match message.tasks {
        TaskSelection::TaskIds(task_ids) => task_action_response_helper(
            "Tasks are being paused",
            task_ids,
            |task| matches!(task.status, TaskStatus::Running),
            &state,
        ),
        TaskSelection::Group(group) => {
            create_success_message(format!("Group \"{group}\" is being paused."))
        }
        TaskSelection::All => create_success_message("All queues are being paused."),
    }
}
