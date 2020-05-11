use ::std::collections::BTreeMap;
use ::std::sync::mpsc::Sender;

use crate::response_helper::*;
use ::pueue::log::{clean_log_handles, read_and_compress_log_files};
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
        Message::Group(message) => group(message, state),

        Message::Clean => clean(state),
        Message::Reset => reset(sender, state),
        Message::Status => get_status(state),
        Message::Log(message) => get_log(message, state),
        Message::Parallel(message) => set_parallel_tasks(message, state),
        _ => create_failure_message("Not implemented yet"),
    }
}

/// Queues a new task to the state.
/// If the start_immediately flag is set, send a StartMessage to the task handler.
/// Invoked when calling `pueue add`.
fn add_task(message: AddMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let starting_status = if message.stashed || message.enqueue_at.is_some() {
        TaskStatus::Stashed
    } else {
        TaskStatus::Queued
    };

    let mut state = state.lock().unwrap();

    // Ensure that specified dependencies actually exist.
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

    // Create a new task and add it to the state.
    let task = Task::new(
        message.command,
        message.path,
        message.group,
        starting_status,
        message.enqueue_at,
        message.dependencies,
    );

    // Create a new group in case the user used a unknown group.
    if let Some(group) = &task.group {
        if state.groups.get(group).is_none() {
            return create_failure_message(format!(
                "Tried to create task with unknown group '{}'",
                group
            ));
        }
    }

    let task_id = state.add_task(task);

    // Notify the task handler, in case the client wants to start the task immediately.
    if message.start_immediately {
        sender
            .send(Message::Start(vec![task_id]))
            .expect(SENDER_ERR);
    }
    // Create the customized response for the client.
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

/// Remove tasks from the queue.
/// We have to ensure that those tasks aren't running!
/// Invoked when calling `pueue remove`.
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

/// Switch the position of two tasks in the upcoming queue.
/// We have to ensure that those tasks are either `Queued` or `Stashed`
/// Invoked when calling `pueue switch`.
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
/// They won't be executed until they're enqueued or explicitely started.
/// Invoked when calling `pueue stash`.
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
    create_success_message(response)
}

/// Enqueue specific stashed tasks.
/// Invoked when calling `pueue enqueue`.
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
    create_success_message(response)
}

/// Forward the start message to the task handler, which then starts the process(es).
/// Invoked when calling `pueue start`.
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

    create_success_message("Daemon and all tasks are being resumed.")
}

/// Create and enqueue tasks from already finished tasks.
/// The user can specify to immediately start the newly created tasks.
/// Invoked when calling `pueue restart`.
fn restart(message: RestartMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    let new_status = if message.stashed {
        TaskStatus::Stashed
    } else {
        TaskStatus::Queued
    };

    let response: String;
    let new_ids = {
        let mut state = state.lock().unwrap();
        let (matching, mismatching) =
            state.tasks_in_statuses(vec![TaskStatus::Done], Some(message.task_ids));

        let mut new_ids = Vec::new();
        for task_id in &matching {
            let task = state.tasks.get(task_id).unwrap();
            let mut new_task = Task::from_task(task);
            new_task.status = new_status.clone();
            new_ids.push(state.add_task(new_task));
        }

        // Already create the response string in here.
        // Otherwise we would need to get matching/mismatching out of this scope.
        response = compile_task_response("Restarted tasks", matching, mismatching);

        new_ids
    };

    // If the restarted tasks should be started immediately, send a message
    // with the new task ids to the task handler.
    if message.start_immediately {
        sender.send(Message::Start(new_ids)).expect(SENDER_ERR);
    }

    create_success_message(response)
}

/// Forward the pause message to the task handler, which then pauses the process(es).
/// Invoked when calling `pueue pause`.
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

    create_success_message("Daemon and all tasks are being paused.")
}

/// Forward the kill message to the task handler, which then kills the process.
/// Invoked when calling `pueue kill`.
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

    create_success_message("All tasks are being killed.")
}

/// The message will be forwarded to the task handler, which then sends the user input to the process.
/// In here we only do some error handling.
/// Invoked when calling `pueue send`.
fn send(message: SendMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
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

/// If a user wants to edit a message, we need to send him the current command.
/// Lock the task to prevent execution, before the user has finished editing the command.
/// Invoked when calling `pueue edit`.
fn edit_request(task_id: usize, state: &SharedState) -> Message {
    // Check whether the task exists and is queued/stashed. Abort if that's not the case.
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
            Message::EditResponse(message)
        }
        None => create_failure_message("No task with this id."),
    }
}

/// Now we actually update the message with the updated command from the client.
/// Invoked after closing the editor on `pueue edit`.
fn edit(message: EditMessage, state: &SharedState) -> Message {
    // Check whether the task exists and is locked. Abort if that's not the case
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

            create_success_message("Command has been updated")
        }
        None => {
            create_failure_message(format!(
                "Task to edit has gone away: {}",
                message.task_id
            ))
        }
    }
}

/// Manage groups.
/// - Show groups
/// - Add group
/// - Remove group
/// Invoked on `pueue groups`.
fn group(message: GroupMessage, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();

    // Create a new group
    if let Some(group) = message.add {
        if state.groups.contains_key(&group) {
            return create_failure_message(format!("Group {} already exists", group));
        }
        if let Err(error) = state.create_group(&group) {
            return create_failure_message(format!(
                "Failed while saving the config file: {}",
                error
            ));
        }

        return create_success_message(format!("Group {} created", group));
    }

    // Remove a new group
    if let Some(group) = message.remove {
        if !state.groups.contains_key(&group) {
            return create_failure_message(format!("Group {} doesn't exists", group));
        }
        if let Err(error) = state.remove_group(&group) {
            return create_failure_message(format!(
                "Failed while saving the config file: {}",
                error
            ));
        }
        return create_success_message(format!("Group {} removed", group));
    }

    // Print all groups
    create_success_message(format!("Groups: {:?}", state.groups.keys()))
}

/// Remove all failed or done tasks from the state.
/// Invoked when calling `pueue clean`.
fn clean(state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    state.backup();
    let (matching, _) = state.tasks_in_statuses(vec![TaskStatus::Done], None);

    for task_id in &matching {
        let _ = state.tasks.remove(task_id).unwrap();
        clean_log_handles(*task_id, &state.settings.daemon.pueue_directory);
    }

    state.save();

    create_success_message("All finished tasks have been removed")
}

/// Forward the reset request to the task handler.
/// The handler then kills all children and clears the task queue.
/// Invoked when calling `pueue reset`.
fn reset(sender: &Sender<Message>, state: &SharedState) -> Message {
    sender.send(Message::Reset).expect(SENDER_ERR);
    clean(state);
    create_success_message("Everything is being reset right now.")
}

/// Return the current state.
/// Invoked when calling `pueue status`.
fn get_status(state: &SharedState) -> Message {
    let state = state.lock().unwrap().clone();
    Message::StatusResponse(state)
}

/// Return the current state and the stdou/stderr of all tasks to the client.
/// Invoked when calling `pueue log`.
fn get_log(message: LogRequestMessage, state: &SharedState) -> Message {
    let state = state.lock().unwrap().clone();
    // Return all logs, if no specific task id is specified
    let task_ids = if message.task_ids.is_empty() {
        state.tasks.keys().cloned().collect()
    } else {
        message.task_ids
    };

    let mut tasks = BTreeMap::new();
    for task_id in task_ids.iter() {
        if let Some(task) = state.tasks.get(task_id) {
            // We send log output and the task at the same time.
            // This isn't as efficient as sending the raw compressed data directly,
            // but it's a lot more convenient for now.
            let (stdout, stderr) = if message.send_logs {
                match read_and_compress_log_files(
                    *task_id,
                    &state.settings.daemon.pueue_directory,
                ) {
                    Ok((stdout, stderr)) => (Some(stdout), Some(stderr)),
                    Err(err) => {
                        return create_failure_message(format!(
                            "Failed reading process output file: {:?}",
                            err
                        ));
                    }
                }
            } else {
                (None, None)
            };

            let task_log = TaskLogMessage {
                task: task.clone(),
                stdout,
                stderr,
            };
            tasks.insert(*task_id, task_log);
        }
    }
    Message::LogResponse(tasks)
}

/// Set the parallel tasks for either a specific group or the global default.
fn set_parallel_tasks(message: ParallelMessage, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();

    // Set the default parallel tasks if no group is specified.
    if message.group.is_none() {
        state.settings.daemon.default_parallel_tasks = message.parallel_tasks;
        return create_success_message("Parallel tasks setting adjusted");
    }

    // We can safely unwrap, since we handled the `None` case above.
    let group = &message.group.unwrap();
    // Check if the given group exists.
    if !state.groups.contains_key(group) {
        return create_failure_message(format!(
            "Unknown group. Use one of these: {:?}",
            state.groups.keys()
        ));
    }

    state
        .settings
        .daemon
        .groups
        .insert(group.into(), message.parallel_tasks);

    if let Err(error) = state.settings.save() {
        return create_failure_message(format!("Failed while saving the config file: {}", error));
    }

    create_success_message(format!(
        "Parallel tasks setting for group {} adjusted",
        group
    ))
}
