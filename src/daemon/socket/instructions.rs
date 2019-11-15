use ::anyhow::Result;
use ::std::sync::mpsc::Sender;

use crate::communication::message::*;
use crate::daemon::state::{SharedState, State};
use crate::daemon::task::Task;

pub fn handle_message(
    message: Message,
    _sender: Sender<Message>,
    state: SharedState,
) -> Result<Message> {
    match message {
        Message::Add(message) => add_task(message, state),
        Message::Status => get_status(state),
        _ => Ok(create_failure_message(String::from("Not implemented yet"))),
    }
}

fn add_task(message: AddMessage, state: SharedState) -> Result<Message> {
    let task = Task::new(message.command, message.arguments, message.path);
    {
        let mut state = state.lock().unwrap();
        state.add_task(task);
    }

    Ok(create_success_message(String::from("New task added.")))
}

fn get_status( state: SharedState) -> Result<Message> {
    let state_clone: State;
    {
        let state = state.lock().unwrap();
        state_clone = state.clone();
    }

    Ok(Message::StatusResponse(state_clone))
}
