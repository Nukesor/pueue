use crossbeam_channel::Sender;

use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use super::SENDER_ERR;

/// Invoked when calling `pueue send`.
/// The message will be forwarded to the task handler, which then sends the user input to the process.
/// In here we only do some error handling.
pub fn send(message: SendMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    // Check whether the task exists and is running. Abort if that's not the case.
    {
        let state = state.lock().unwrap();
        match state.tasks.get(&message.task_id) {
            Some(task) => {
                if task.status != TaskStatus::Running {
                    return create_failure_message("You can only send input to a running task");
                }
            }
            None => return create_failure_message("No task with this id."),
        }
    }

    // Check whether the task exists and is running, abort if that's not the case.
    sender.send(Message::Send(message)).expect(SENDER_ERR);

    create_success_message("Message is being send to the process.")
}
