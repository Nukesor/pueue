use crossbeam_channel::Sender;

use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;

use super::SENDER_ERR;
use crate::network::response_helper::{ensure_group_exists, task_action_response_helper};

/// Invoked when calling `pueue kill`.
/// Forward the kill message to the task handler, which then kills the process.
pub fn kill(message: KillMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();

    // If a group is selected, make sure it exists.
    if let TaskSelection::Group(group) = &message.tasks {
        if let Err(message) = ensure_group_exists(&mut state, group) {
            return message;
        }
    }

    sender
        .send(Message::Kill(message.clone()))
        .expect(SENDER_ERR);

    if let Some(signal) = message.signal {
        match message.tasks {
            TaskSelection::TaskIds(task_ids) => task_action_response_helper(
                "Tasks are being killed",
                task_ids,
                |task| task.is_running(),
                &state,
            ),
            TaskSelection::Group(group) => create_success_message(format!(
                "Sending signal {signal} to all running tasks of group {group}.",
            )),
            TaskSelection::All => {
                create_success_message(format!("Sending signal {signal} to all running tasks."))
            }
        }
    } else {
        match message.tasks {
            TaskSelection::TaskIds(task_ids) => task_action_response_helper(
                "Tasks are being killed",
                task_ids,
                |task| task.is_running(),
                &state,
            ),
            TaskSelection::Group(group) => create_success_message(format!(
                "All tasks of group \"{group}\" are being killed. The group will also be paused!!!"
            )),
            TaskSelection::All => {
                create_success_message("All tasks are being killed. All groups will be paused!!!")
            }
        }
    }
}
