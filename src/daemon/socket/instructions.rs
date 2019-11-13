use ::anyhow::Result;
use ::std::sync::mpsc::Sender;

use crate::daemon::state::SharedState;
use crate::communication::message::*;
use crate::daemon::task::Task;


pub fn handle_message(message: Message, _sender: Sender<Message>, state: SharedState) -> Result<Message> {
    match message {
        Message::Add(message) => add_task(message, state),
        _ => Ok(create_failure_message(String::from("Not implemented yet")))
    }
}


fn add_task(message: AddMessage, state: SharedState) -> Result<Message> {
    let mut state = state.lock().unwrap();

    let task = Task::new(message.command, message.arguments, message.path);

    state.add_task(task);

    Ok(create_success_message(String::from("New task added.")))
}
