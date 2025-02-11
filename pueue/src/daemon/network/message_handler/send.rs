use std::io::Write;

use pueue_lib::{failure_msg, network::message::*};

use crate::daemon::internal_state::SharedState;

/// Invoked when calling `pueue send`.
/// The message will be forwarded to the task handler, which then sends the user input to the
/// process. In here we only do some error handling.
pub fn send(state: &SharedState, message: SendMessage) -> Response {
    let task_id = message.task_id;
    let mut state = state.lock().unwrap();

    // Check whether the task exists and is running. Abort if that's not the case.
    let child = match state.children.get_child_mut(task_id) {
        Some(child) => child,
        None => {
            return failure_msg!("You can only send input to a running process.");
        }
    };
    {
        let child_stdin = child.inner().stdin.as_mut().unwrap();
        if let Err(err) = child_stdin.write_all(&message.input.into_bytes()) {
            return failure_msg!("Failed to send input to task {task_id} with err {err:?}");
        };
    }

    create_success_response("Message is being send to the process.")
}
