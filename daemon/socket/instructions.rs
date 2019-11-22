use ::std::sync::mpsc::Sender;

use ::pueue::message::*;
use ::pueue::state::SharedState;
use ::pueue::task::{Task, TaskStatus};

static SENDER_ERR: &str = "Failed to send message to task handler thread";

pub fn handle_message(message: Message, sender: Sender<Message>, state: SharedState) -> Message {
    match message {
        Message::Add(message) => add_task(message, sender, state),
        Message::Remove(message) => remove(message, state),
        Message::Start(message) => start(message, sender, state),
        Message::Restart(message) => restart(message, sender, state),

        Message::Pause(message) => pause(message, sender, state),
        Message::Stash(message) => stash(message, state),
        Message::Enqueue(message) => enqueue(message, state),
        Message::Kill(message) => kill(message, sender, state),

        Message::Send(message) => send(message, sender, state),

        Message::Clean => clean(state),
        Message::Reset => reset(sender),
        Message::Status => get_status(state),
        _ => create_failure_message("Not implemented yet"),
    }
}

/// Queues a new task to the state.
/// If the start_immediately flag is set, send a StartMessage to the task handler
fn add_task(message: AddMessage, sender: Sender<Message>, state: SharedState) -> Message {
    let task = Task::new(message.command, message.path);
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

    create_success_message("New task added.")
}

/// Remove tasks from the queue
fn remove(message: RemoveMessage, state: SharedState) -> Message {
    let (matching, mismatching) = {
        let statuses = vec![TaskStatus::Running, TaskStatus::Paused];
        let mut state = state.lock().unwrap();
        state.tasks_not_in_statuses(Some(message.task_ids), statuses)
    };

    let message = "Tasks removed from list";
    let message = compile_task_response(message, matching, mismatching);
    create_success_message(message)
}

/// Simply return the current state to the client
fn get_status(state: SharedState) -> Message {
    let state = state.lock().unwrap();
    Message::StatusResponse(state.clone())
}

/// Forward the start message to the task handler and respond to the client
fn start(message: StartMessage, sender: Sender<Message>, state: SharedState) -> Message {
    sender
        .send(Message::Start(message.clone()))
        .expect(SENDER_ERR);
    if let Some(task_ids) = message.task_ids {
        let response = task_response_helper(
            "Tasks are being started",
            task_ids,
            vec![TaskStatus::Paused, TaskStatus::Queued, TaskStatus::Stashed],
            state,
        );
        return create_success_message(response);
    }

    return create_success_message("Daemon and all tasks are being resumed.");
}

/// Create and enqueue tasks from already finished tasks
/// The user can specify to immediately start the newly created tasks.
fn restart(message: RestartMessage, sender: Sender<Message>, state: SharedState) -> Message {
    let response: String;
    let new_ids = {
        let mut state = state.lock().unwrap();
        let statuses = vec![TaskStatus::Done, TaskStatus::Failed];
        let (matching, mismatching) = state.tasks_in_statuses(Some(message.task_ids), statuses);

        let mut new_ids = Vec::new();
        for task_id in &matching {
            let task = state.tasks.get(task_id).unwrap();
            let new_task = Task::from_task(task);
            new_ids.push(state.add_task(new_task));
        }

        // Already create the response string in here. Otherwise we would
        // need to get matching/mismatching out of this scope
        let message = "Restarted tasks";
        response = compile_task_response(message, matching, mismatching);

        new_ids
    };

    // If the restarted tasks should be started immediately, send a message
    // with the new task ids to the task handler.
    if message.start_immediately {
        let start_message = StartMessage {
            task_ids: Some(new_ids),
        };
        sender
            .send(Message::Start(start_message))
            .expect(SENDER_ERR);
    }

    return create_success_message(response);
}

/// Forward the pause message to the task handler and respond to the client
fn pause(message: PauseMessage, sender: Sender<Message>, state: SharedState) -> Message {
    sender
        .send(Message::Pause(message.clone()))
        .expect(SENDER_ERR);
    if let Some(task_ids) = message.task_ids {
        let response = task_response_helper(
            "Tasks are being paused",
            task_ids,
            vec![TaskStatus::Running],
            state,
        );
        return create_success_message(response);
    }

    return create_success_message("Daemon and all tasks are being paused.");
}

/// Stash specific queued tasks.
/// They won't be executed until enqueued again or explicitely started
fn stash(message: StashMessage, state: SharedState) -> Message {
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        let (matching, mismatching) = state.tasks_in_statuses(Some(message.task_ids), vec![TaskStatus::Queued]);

        for task_id in &matching {
            state.change_status(*task_id, TaskStatus::Stashed);
        }

        (matching, mismatching)
    };

    let message = "Tasks are stashed";
    let response = compile_task_response(message, matching, mismatching);
    return create_success_message(response);
}


/// Enqueue specific stashed tasks.
/// They will be normally handled afterwards.
fn enqueue(message: EnqueueMessage, state: SharedState) -> Message {
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        let (matching, mismatching) = state.tasks_in_statuses(Some(message.task_ids), vec![TaskStatus::Stashed]);

        for task_id in &matching {
            state.change_status(*task_id, TaskStatus::Queued);
        }

        (matching, mismatching)
    };

    let message = "Tasks are enqueued";
    let response = compile_task_response(message, matching, mismatching);
    return create_success_message(response);
}


/// Forward the kill message to the task handler and respond to the client
fn kill(message: KillMessage, sender: Sender<Message>, state: SharedState) -> Message {
    sender
        .send(Message::Kill(message.clone()))
        .expect(SENDER_ERR);

    if !message.task_ids.is_empty() {
        let response = task_response_helper(
            "Tasks are being killed",
            message.task_ids,
            vec![TaskStatus::Running, TaskStatus::Paused],
            state,
        );
        return create_success_message(response);
    }

    return create_success_message("All tasks are being killed.");
}

// Send some user defined input to a process
// The message will be forwarded to the task handler.
// In here we only do some error handling.
fn send(message: SendMessage, sender: Sender<Message>, state: SharedState) -> Message {
    // Check whether the task exists and is running, abort if that's not the case
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

    // Check whether the task exists and is running, abort if that's not the case
    sender
        .send(Message::Send(message))
        .expect(SENDER_ERR);

    return create_success_message("Message is being send to the process.");
}

/// Remove all failed or done tasks from the state
fn clean(state: SharedState) -> Message {
    let mut state = state.lock().unwrap();
    let statuses = vec![TaskStatus::Done, TaskStatus::Failed];
    let (matching, _) = state.tasks_in_statuses(None, statuses);

    for task_id in &matching {
        let _ = state.tasks.remove(task_id).unwrap();
    }

    return create_success_message("All finished tasks have been removed");
}

// Forward the reset request to the task handler
// The handler then kills all children and clears the task queue
fn reset(sender: Sender<Message>) -> Message {
    sender.send(Message::Reset).expect(SENDER_ERR);
    return create_success_message("Everything is being reset right now.");
}

fn task_response_helper(
    message: &'static str,
    task_ids: Vec<i32>,
    statuses: Vec<TaskStatus>,
    state: SharedState,
) -> String {
    // Get all matching/mismatching task_ids for all given ids and statuses
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        state.tasks_in_statuses(Some(task_ids), statuses)
    };

    compile_task_response(message, matching, mismatching)
}

/// Compile a response for instructions with multiple tasks ids
/// A custom message will be combined with a text about all matching tasks
/// and possibly tasks for which the instruction cannot be executed.
fn compile_task_response(
    message: &'static str,
    matching: Vec<i32>,
    mismatching: Vec<i32>,
) -> String {
    let matching: Vec<String> = matching.iter().map(|id| id.to_string()).collect();
    let mismatching: Vec<String> = mismatching.iter().map(|id| id.to_string()).collect();
    let matching_string = matching.join(", ");

    // We don't have any mismatching ids, return the simple message
    if mismatching.is_empty() {
        return format!("{}: {}", message, matching_string);
    }

    let mismatched_message = "The command couldn't be executed for these tasks";
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
