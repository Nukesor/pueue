use ::anyhow::Result;
use ::std::sync::mpsc::Sender;

use ::pueue::communication::message::*;
use ::pueue::state::{SharedState, State};
use ::pueue::task::Task;

pub fn handle_message(
    message: Message,
    sender: Sender<Message>,
    state: SharedState,
) -> Message {
    match message {
        Message::Add(message) => add_task(message, state),
        Message::Pause(message) => pause(message, sender),
        Message::Start(message) => start(message, sender),
        Message::Status => get_status(state),
        _ => create_failure_message(String::from("Not implemented yet")),
    }
}

fn add_task(message: AddMessage, state: SharedState) -> Message {
    let task = Task::new(message.command, message.arguments, message.path);
    {
        let mut state = state.lock().unwrap();
        state.add_task(task);
    }

    create_success_message(String::from("New task added."))
}

fn get_status( state: SharedState) -> Message {
    let state_clone: State;
    {
        let state = state.lock().unwrap();
        state_clone = state.clone();
    }

    Message::StatusResponse(state_clone)
}

fn pause(message: PauseMessage, sender: Sender<Message>) -> Message {
    sender.send(Message::Pause(message.clone())).expect("Failed to send to message to task handler thread");
    if let Some(_) = message.task_ids {
        return create_success_message(String::from("Specified tasks are being paused."));
    }

    return create_success_message(String::from("Daemon and all tasks are being paused."));
}

fn start(message: StartMessage, sender: Sender<Message>) -> Message {
    sender.send(Message::Start(message.clone())).expect("Failed to send to message to task handler thread");
    if let Some(_) = message.task_ids {
        return create_success_message(String::from("Specified tasks are being started."));
    }

    return create_success_message(String::from("Daemon and all tasks are being resumed."));
}
