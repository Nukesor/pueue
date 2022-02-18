use crossbeam_channel::Sender;

use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use super::SENDER_ERR;
use crate::network::response_helper::*;

/// Invoked when calling `pueue start`.
/// Forward the start message to the task handler, which then starts the process(es).
pub fn start(message: StartMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(message) = ensure_group_exists(&mut state, group) {
            return message;
        }
    }

    // Forward the message to the task handler.
    sender
        .send(Message::Start(message.clone()))
        .expect(SENDER_ERR);

    // Return a response depending on the selected tasks.
    match message.tasks {
        TaskSelection::TaskIds(task_ids) => task_action_response_helper(
            "Tasks are being started",
            task_ids,
            |task| {
                matches!(
                    task.status,
                    TaskStatus::Paused | TaskStatus::Queued | TaskStatus::Stashed { .. }
                )
            },
            &state,
        ),
        TaskSelection::Group(group) => {
            create_success_message(format!("Group \"{group}\" is being resumed."))
        }
        TaskSelection::All => create_success_message("All queues are being resumed."),
    }
}
