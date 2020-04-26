use ::std::collections::BTreeMap;
use ::std::sync::mpsc::Sender;

use crate::response_helper::*;
use ::pueue::message::*;
use ::pueue::state::SharedState;
use ::pueue::task::{Task, TaskStatus};

static SENDER_ERR: &str = "Failed to send message to task handler thread";

pub fn handle_message(message: Message, sender: &Sender<Message>, state: &SharedState) -> Message {
    match message {
        Message::Add(message) => add_task(message, sender, state),
        Message::Remove(task_ids) => remove(task_ids, state),
        Message::Switch(message) => switch(message, state),
        Message::Stash(task_ids) => stash(task_ids, state),
        Message::Enqueue(message) => enqueue(message, state),

        Message::Start(task_ids) => start(task_ids, sender, state),
        Message::Restart(message) => restart(message, sender, state),
        Message::Pause(message) => pause(message, sender, state),
        Message::Kill(message) => kill(message, sender, state),

        Message::Send(message) => send(message, sender, state),
        Message::EditRequest(task_id) => edit_request(task_id, state),
        Message::Edit(message) => edit(message, state),

        Message::Clean => clean(state),
        Message::Reset => reset(sender),
        Message::Status => get_status(state),
        Message::Log(task_ids) => get_log(task_ids, state),
        Message::Parallel(amount) => set_parallel_tasks(amount, sender),
        _ => create_failure_message("Not implemented yet"),
    }
}

/// Queues a new task to the state.
/// If the start_immediately flag is set, send a StartMessage to the task handler
fn add_task(message: AddMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let starting_status = if message.stashed || message.enqueue_at.is_some() {
        TaskStatus::Stashed
    } else {
        TaskStatus::Queued
    };

    let mut state = state.lock().unwrap();

    let not_found: Vec<_> = message
        .dependencies
        .iter()
        .filter(|id| !state.tasks.contains_key(id))
        .collect();

    if !not_found.is_empty() {
        return create_failure_message(format!(
            "Unable to setup dependencies : task(s) {:?} not found",
            not_found
        ));
    }
    let task = Task::new(
        message.command,
        message.path,
        starting_status,
        message.enqueue_at,
        message.dependencies,
    );
    let task_id = state.add_task(task);
    if message.start_immediately {
        sender
            .send(Message::Start(vec![task_id]))
            .expect(SENDER_ERR);
    }

    let message = if let Some(enqueue_at) = message.enqueue_at {
        format!(
            "New task added (id {}). It will be enqueued at {}",
            task_id,
            enqueue_at.format("%Y-%m-%d %H:%M:%S")
        )
    } else {
        format!("New task added (id {}).", task_id)
    };
    state.save();

    create_success_message(message)
}

/// Remove tasks from the queue
fn remove(task_ids: Vec<usize>, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    let statuses = vec![TaskStatus::Running, TaskStatus::Paused];
    let (running, not_running) = state.tasks_in_statuses(statuses, Some(task_ids));
    println!("{:?}", not_running);

    for task_id in &not_running {
        state.tasks.remove(task_id);
    }

    let text = "Tasks removed from list";
    let response = compile_task_response(text, not_running, running);
    create_success_message(response)
}

/// Switch the position of two tasks in the upcoming queue
fn switch(message: SwitchMessage, state: &SharedState) -> Message {
    let task_ids = vec![message.task_id_1, message.task_id_2];
    let statuses = vec![TaskStatus::Queued, TaskStatus::Stashed];
    let mut state = state.lock().unwrap();
    let (_, mismatching) = state.tasks_in_statuses(statuses, Some(task_ids.clone().to_vec()));
    if !mismatching.is_empty() {
        return create_failure_message("Tasks have to be either queued or stashed.");
    }

    // Get the tasks. Expect them to be there, since we found no mismatch
    let mut first_task = state.tasks.remove(&task_ids[0]).unwrap();
    let mut second_task = state.tasks.remove(&task_ids[1]).unwrap();

    // Switch task ids
    let temp_id = first_task.id;
    first_task.id = second_task.id;
    second_task.id = temp_id;

    // Put tasks back in again
    state.tasks.insert(first_task.id, first_task);
    state.tasks.insert(second_task.id, second_task);

    create_success_message("Tasks have been switched")
}

/// Stash specific queued tasks.
/// They won't be executed until enqueued again or explicitely started
fn stash(task_ids: Vec<usize>, state: &SharedState) -> Message {
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        let (matching, mismatching) =
            state.tasks_in_statuses(vec![TaskStatus::Queued, TaskStatus::Locked], Some(task_ids));

        for task_id in &matching {
            state.change_status(*task_id, TaskStatus::Stashed);
        }

        (matching, mismatching)
    };

    let text = "Tasks are stashed";
    let response = compile_task_response(text, matching, mismatching);
    return create_success_message(response);
}

/// Enqueue specific stashed tasks.
/// They will be normally handled afterwards.
fn enqueue(message: EnqueueMessage, state: &SharedState) -> Message {
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        let (matching, mismatching) = state.tasks_in_statuses(
            vec![TaskStatus::Stashed, TaskStatus::Locked],
            Some(message.task_ids),
        );

        for task_id in &matching {
            state.set_enqueue_at(*task_id, message.enqueue_at);
            state.change_status(*task_id, TaskStatus::Queued);
        }

        (matching, mismatching)
    };

    let text = if let Some(enqueue_at) = message.enqueue_at {
        format!(
            "Tasks will be enqueued at {}",
            enqueue_at.format("%Y-%m-%d %H:%M:%S")
        )
    } else {
        String::from("Tasks are enqueued")
    };

    let response = compile_task_response(text.as_str(), matching, mismatching);
    return create_success_message(response);
}

/// Forward the start message to the task handler and respond to the client
fn start(task_ids: Vec<usize>, sender: &Sender<Message>, state: &SharedState) -> Message {
    sender
        .send(Message::Start(task_ids.clone()))
        .expect(SENDER_ERR);
    if !task_ids.is_empty() {
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
fn restart(message: RestartMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let new_status = if message.stashed {
        TaskStatus::Stashed
    } else {
        TaskStatus::Queued
    };

    let response: String;
    let new_ids = {
        let mut state = state.lock().unwrap();
        let statuses = vec![TaskStatus::Done, TaskStatus::Failed, TaskStatus::Killed];
        let (matching, mismatching) = state.tasks_in_statuses(statuses, Some(message.task_ids));

        let mut new_ids = Vec::new();
        for task_id in &matching {
            let task = state.tasks.get(task_id).unwrap();
            let mut new_task = Task::from_task(task);
            new_task.status = new_status.clone();
            new_ids.push(state.add_task(new_task));
        }

        // Already create the response string in here. Otherwise we would
        // need to get matching/mismatching out of this scope
        response = compile_task_response("Restarted tasks", matching, mismatching);

        new_ids
    };

    // If the restarted tasks should be started immediately, send a message
    // with the new task ids to the task handler.
    if message.start_immediately {
        sender.send(Message::Start(new_ids)).expect(SENDER_ERR);
    }

    return create_success_message(response);
}

/// Forward the pause message to the task handler and respond to the client
fn pause(message: PauseMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    sender
        .send(Message::Pause(message.clone()))
        .expect(SENDER_ERR);
    if !message.task_ids.is_empty() {
        let response = task_response_helper(
            "Tasks are being paused",
            message.task_ids,
            vec![TaskStatus::Running],
            state,
        );
        return create_success_message(response);
    }

    return create_success_message("Daemon and all tasks are being paused.");
}

/// Forward the kill message to the task handler and respond to the client
fn kill(message: KillMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
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
fn send(message: SendMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
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
    sender.send(Message::Send(message)).expect(SENDER_ERR);

    return create_success_message("Message is being send to the process.");
}

// If a user wants to edit a message, we need to send him the current command
// and lock the task to prevent execution, before the user has finished editing the command
fn edit_request(task_id: usize, state: &SharedState) -> Message {
    // Check whether the task exists and is queued/stashed, abort if that's not the case
    let mut state = state.lock().unwrap();
    match state.tasks.get_mut(&task_id) {
        Some(task) => {
            if !task.is_queued() {
                return create_failure_message("You can only edit a queued/stashed task");
            }
            task.prev_status = task.status.clone();
            task.status = TaskStatus::Locked;

            let message = EditResponseMessage {
                task_id: task.id,
                command: task.command.clone(),
                path: task.path.clone(),
            };
            return Message::EditResponse(message);
        }
        None => return create_failure_message("No task with this id."),
    }
}

// Handle the actual updated command
fn edit(message: EditMessage, state: &SharedState) -> Message {
    // Check whether the task exists and is queued/stashed, abort if that's not the case
    let mut state = state.lock().unwrap();
    match state.tasks.get_mut(&message.task_id) {
        Some(task) => {
            if !(task.status == TaskStatus::Locked) {
                return create_failure_message("Task is no longer locked.");
            }

            task.status = task.prev_status.clone();
            task.command = message.command.clone();
            task.path = message.path.clone();
            state.save();

            return create_success_message("Command has been updated");
        }
        None => {
            return create_failure_message(format!(
                "Task to edit has gone away: {}",
                message.task_id
            ))
        }
    }
}

/// Remove all failed or done tasks from the state
fn clean(state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    state.clean();

    return create_success_message("All finished tasks have been removed");
}

// Forward the reset request to the task handler
// The handler then kills all children and clears the task queue
fn reset(sender: &Sender<Message>) -> Message {
    sender.send(Message::Reset).expect(SENDER_ERR);
    return create_success_message("Everything is being reset right now.");
}

/// Return the current state without any stdou/stderr to the client
/// Invoked when calling `pueue status`
fn get_status(state: &SharedState) -> Message {
    let mut state = { state.lock().unwrap().clone() };
    for (_, task) in state.tasks.iter_mut() {
        task.stdout = None;
        task.stderr = None;
    }
    Message::StatusResponse(state)
}

/// Return the current state without any stdou/stderr to the client
/// Invoked when calling `pueue log`
fn get_log(task_ids: Vec<usize>, state: &SharedState) -> Message {
    let state = state.lock().unwrap().clone();
    // Return all logs, if no specific task id is specified
    if task_ids.is_empty() {
        return Message::LogResponse(state.tasks.clone());
    }

    let mut tasks = BTreeMap::new();
    for task_id in task_ids.iter() {
        match state.tasks.get(task_id) {
            Some(task) => {
                tasks.insert(*task_id, task.clone());
            }
            None => {}
        }
    }
    Message::LogResponse(tasks)
}

fn set_parallel_tasks(amount: usize, sender: &Sender<Message>) -> Message {
    sender.send(Message::Parallel(amount)).expect(SENDER_ERR);
    return create_success_message("Parallel tasks setting adjusted");
}
