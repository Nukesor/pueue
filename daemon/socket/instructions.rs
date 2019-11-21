use ::std::sync::mpsc::Sender;

use ::pueue::communication::message::*;
use ::pueue::state::{SharedState, State};
use ::pueue::task::{Task, TaskStatus};

static SENDER_ERR: &str = "Failed to send message to task handler thread";

pub fn handle_message(message: Message, sender: Sender<Message>, state: SharedState) -> Message {
    match message {
        Message::Add(message) => add_task(message, sender, state),
        Message::Pause(message) => pause(message, sender, state),
        Message::Start(message) => start(message, sender, state),
        Message::Status => get_status(state),
        _ => create_failure_message(String::from("Not implemented yet")),
    }
}

fn add_task(message: AddMessage, sender: Sender<Message>, state: SharedState) -> Message {
    let task = Task::new(message.command, message.arguments, message.path);
    let task_id: i32;
    {
        let mut state = state.lock().unwrap();
        task_id = state.add_task(task);
    }
    if message.start_immediately {
        let start_message = StartMessage {
            task_ids: Some(vec![task_id]),
        };
        sender
            .send(Message::Start(start_message))
            .expect(SENDER_ERR);
    }

    create_success_message(String::from("New task added."))
}

fn get_status(state: SharedState) -> Message {
    let state_clone: State;
    {
        let state = state.lock().unwrap();
        state_clone = state.clone();
    }

    Message::StatusResponse(state_clone)
}

/// Forward the pause message to the task handler and respond to the client
fn pause(message: PauseMessage, sender: Sender<Message>, state: SharedState) -> Message {
    sender
        .send(Message::Pause(message.clone()))
        .expect(SENDER_ERR);
    if let Some(task_ids) = message.task_ids {
        let response = compile_task_response(
            "Specified tasks are being paused",
            task_ids,
            vec![TaskStatus::Running],
            state,
        );
        return create_success_message(response);
    }

    return create_success_message(String::from("Daemon and all tasks are being paused."));
}

/// Forward the start message to the task handler and respond to the client
fn start(message: StartMessage, sender: Sender<Message>, state: SharedState) -> Message {
    sender
        .send(Message::Start(message.clone()))
        .expect(SENDER_ERR);
    if let Some(task_ids) = message.task_ids {
        let response = compile_task_response(
            "Specified tasks are being started",
            task_ids,
            vec![TaskStatus::Paused, TaskStatus::Queued, TaskStatus::Stashed],
            state,
        );
        return create_success_message(response);
    }

    return create_success_message(String::from("Daemon and all tasks are being resumed."));
}

/// Compile a response for instructions with multiple tasks ids
/// A custom message will be combined with a text about all matching tasks
/// and possibly tasks for which the instruction cannot be executed.
fn compile_task_response(
    message: &'static str,
    task_ids: Vec<i32>,
    stati: Vec<TaskStatus>,
    state: SharedState,
) -> String {
    // Get all matching/mismatching task_ids for all given ids and stati
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        state.tasks_in_states(Some(task_ids), stati)
    };

    let matching: Vec<String> = matching.iter().map(|id| id.to_string()).collect();
    let mismatching: Vec<String> = mismatching.iter().map(|id| id.to_string()).collect();
    let matching_string = matching.join(", ");

    // We don't have any mismatching ids, return the simple message
    if mismatching.is_empty() {
        return format!("{}: {}", message, matching_string);
    }

    let mismatched_message = "The command couldn't be executed for these ids";
    let mismatching_string = mismatching.join(", ");

    // All given ids are invalid
    if matching.is_empty() {
        return format!("{}: {}", mismatched_message, mismatching_string);
    }

    // Some ids were valid, some were invalid
    format!(
        "{}: {}\n{}: {}",
        message, matching_string, mismatched_message, mismatching_string
    )
}
