use std::sync::mpsc::Sender;
use std::{collections::BTreeMap, sync::MutexGuard};

use log::debug;

use pueue::aliasing::insert_alias;
use pueue::log::{clean_log_handles, read_and_compress_log_files};
use pueue::message::*;
use pueue::state::{SharedState, State};
use pueue::task::{Task, TaskStatus};

use crate::response_helper::*;

static SENDER_ERR: &str = "Failed to send message to task handler thread";

pub fn handle_message(message: Message, sender: &Sender<Message>, state: &SharedState) -> Message {
    match message {
        Message::Add(message) => add_task(message, sender, state),
        Message::Remove(task_ids) => remove(task_ids, state),
        Message::Restart(message) => restart_multiple(message, sender, state),
        Message::Switch(message) => switch(message, state),
        Message::Stash(task_ids) => stash(task_ids, state),
        Message::Enqueue(message) => enqueue(message, state),

        Message::Start(message) => start(message, sender, state),
        Message::Pause(message) => pause(message, sender, state),
        Message::Kill(message) => kill(message, sender, state),

        Message::Send(message) => send(message, sender, state),
        Message::EditRequest(task_id) => edit_request(task_id, state),
        Message::Edit(message) => edit(message, state),
        Message::Group(message) => group(message, state),

        Message::Clean => clean(state),
        Message::Reset(children) => reset(sender, children),
        Message::Status => get_status(state),
        Message::Log(message) => get_log(message, state),
        Message::Parallel(message) => set_parallel_tasks(message, state),
        Message::DaemonShutdown => shutdown(sender, state),
        _ => create_failure_message("Not implemented yet"),
    }
}

/// Invoked when calling `pueue add`.
/// Queues a new task to the state.
/// If the start_immediately flag is set, send a StartMessage to the task handler.
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
        message.envs,
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
            .send(Message::Start(StartMessage {
                task_ids: vec![task_id],
                ..Default::default()
            }))
            .expect(SENDER_ERR);
    }
    // Create the customized response for the client.
    let message = if message.print_task_id {
        task_id.to_string()
    } else if let Some(enqueue_at) = message.enqueue_at {
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

/// Invoked when calling `pueue remove`.
/// Remove tasks from the queue.
/// We have to ensure that those tasks aren't running!
fn remove(task_ids: Vec<usize>, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    let statuses = vec![
        TaskStatus::Queued,
        TaskStatus::Stashed,
        TaskStatus::Done,
        TaskStatus::Locked,
    ];
    let (not_running, running) = state.tasks_in_statuses(statuses, Some(task_ids));

    for task_id in &not_running {
        state.tasks.remove(task_id);
    }

    let text = "Tasks removed from list";
    let response = compile_task_response(text, not_running, running);
    create_success_message(response)
}

/// Invoked when calling `pueue switch`.
/// Switch the position of two tasks in the upcoming queue.
/// We have to ensure that those tasks are either `Queued` or `Stashed`
fn switch(message: SwitchMessage, state: &SharedState) -> Message {
    let task_ids = vec![message.task_id_1, message.task_id_2];
    let statuses = vec![TaskStatus::Queued, TaskStatus::Stashed];
    let mut state = state.lock().unwrap();
    let (_, mismatching) = state.tasks_in_statuses(statuses, Some(task_ids.to_vec()));
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

/// Invoked when calling `pueue stash`.
/// Stash specific queued tasks.
/// They won't be executed until they're enqueued or explicitely started.
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

/// Invoked when calling `pueue enqueue`.
/// Enqueue specific stashed tasks.
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

/// Invoked when calling `pueue start`.
/// Forward the start message to the task handler, which then starts the process(es).
fn start(message: StartMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    // Check whether a given group exists
    if let Some(group) = &message.group {
        let state = state.lock().unwrap();
        if !state.groups.contains_key(group) {
            return create_failure_message(format!("Group {} doesn't exists", group));
        }
    }

    sender
        .send(Message::Start(message.clone()))
        .expect(SENDER_ERR);
    if !message.task_ids.is_empty() {
        let response = task_response_helper(
            "Tasks are being started",
            message.task_ids,
            vec![TaskStatus::Paused, TaskStatus::Queued, TaskStatus::Stashed],
            state,
        );
        return create_success_message(response);
    }

    if let Some(group) = &message.group {
        create_success_message(format!("Group {} is being resumed.", group))
    } else if message.all {
        create_success_message("All queues are being resumed.")
    } else {
        create_success_message("Default queue is being resumed.")
    }
}

/// This is a small wrapper around the actual in-place task `restart` functionality.
fn restart_multiple(
    message: RestartMessage,
    sender: &Sender<Message>,
    state: &SharedState,
) -> Message {
    let mut state = state.lock().unwrap();
    for task in message.tasks.iter() {
        restart(&mut state, task, message.stashed);
    }

    // Tell the task manager to start the task immediately, if it's requested.
    if message.start_immediately {
        sender
            .send(Message::Start(StartMessage {
                task_ids: message.tasks.iter().map(|task| task.task_id).collect(),
                ..Default::default()
            }))
            .expect(SENDER_ERR);
    }

    create_success_message("Tasks restarted")
}

/// This is invoked, whenever a task is actually restarted (in-place) without creating a new task.
/// Update a possibly changed path/command and reset all infos from the previous run.
fn restart(state: &mut MutexGuard<State>, to_restart: &TasksToRestart, stashed: bool) {
    // Check if we actually know this task.
    let task = if let Some(task) = state.tasks.get_mut(&to_restart.task_id) {
        task
    } else {
        return;
    };

    // Either enqueue the task or stash it.
    task.status = if stashed {
        TaskStatus::Stashed
    } else {
        TaskStatus::Queued
    };

    // Update command and path.
    task.original_command = to_restart.command.clone();
    task.command = insert_alias(to_restart.command.clone());
    task.path = to_restart.path.clone();

    // Reset all variables of any previous run.
    task.result = None;
    task.start = None;
    task.end = None;
}

/// Invoked when calling `pueue pause`.
/// Forward the pause message to the task handler, which then pauses groups/tasks/everything.
fn pause(message: PauseMessage, sender: &Sender<Message>, state: &SharedState) -> Message {
    // Check whether a given group exists
    if let Some(group) = &message.group {
        let state = state.lock().unwrap();
        if !state.groups.contains_key(group) {
            return create_failure_message(format!("Group {} doesn't exists", group));
        }
    }

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
    if let Some(group) = &message.group {
        create_success_message(format!("Group {} is being paused.", group))
    } else if message.all {
        create_success_message("All queues are being paused.")
    } else {
        create_success_message("Default queue is being paused.")
    }
}

/// Invoked when calling `pueue kill`.
/// Forward the kill message to the task handler, which then kills the process.
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

    if let Some(group) = &message.group {
        create_success_message(format!("All tasks of Group {} is being killed.", group))
    } else if message.all {
        create_success_message("All tasks are being killed.")
    } else {
        create_success_message("All tasks of the default queue are being paused.")
    }
}

/// Invoked when calling `pueue send`.
/// The message will be forwarded to the task handler, which then sends the user input to the process.
/// In here we only do some error handling.
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

/// Invoked when calling `pueue edit`.
/// If a user wants to edit a message, we need to send him the current command.
/// Lock the task to prevent execution, before the user has finished editing the command.
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
                command: task.original_command.clone(),
                path: task.path.clone(),
            };
            Message::EditResponse(message)
        }
        None => create_failure_message("No task with this id."),
    }
}

/// Invoked after closing the editor on `pueue edit`.
/// Now we actually update the message with the updated command from the client.
fn edit(message: EditMessage, state: &SharedState) -> Message {
    // Check whether the task exists and is locked. Abort if that's not the case
    let mut state = state.lock().unwrap();
    match state.tasks.get_mut(&message.task_id) {
        Some(task) => {
            if !(task.status == TaskStatus::Locked) {
                return create_failure_message("Task is no longer locked.");
            }

            task.status = task.prev_status.clone();
            task.original_command = message.command.clone();
            task.command = insert_alias(message.command.clone());
            task.path = message.path.clone();
            state.save();

            create_success_message("Command has been updated")
        }
        None => create_failure_message(format!("Task to edit has gone away: {}", message.task_id)),
    }
}

/// Invoked on `pueue groups`.
/// Manage groups.
/// - Show groups
/// - Add group
/// - Remove group
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

    // There are no groups yet.
    if state.groups.is_empty() {
        return create_success_message(
            "There are no groups yet. You can add them with the 'group -a' flag",
        );
    }

    // Compile a small minimalistic text with all important information about all known groups
    let mut group_status = String::new();
    let mut group_iter = state.groups.iter().peekable();
    while let Some((group, running)) = group_iter.next() {
        group_status.push_str(&format!(
            "Group {} ({} parallel), running: {}",
            group,
            state.settings.daemon.groups.get(group).unwrap(),
            running
        ));
        if group_iter.peek().is_some() {
            group_status.push('\n');
        }
    }
    create_success_message(group_status)
}

/// Invoked when calling `pueue clean`.
/// Remove all failed or done tasks from the state.
fn clean(state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    state.backup();
    let (matching, _) = state.tasks_in_statuses(vec![TaskStatus::Done], None);

    for task_id in &matching {
        let _ = state.tasks.remove(task_id).unwrap();
        clean_log_handles(*task_id, &state.settings.shared.pueue_directory);
    }

    state.save();

    create_success_message("All finished tasks have been removed")
}

/// Invoked when calling `pueue reset`.
/// Forward the reset request to the task handler.
/// The handler then kills all children and clears the task queue.
fn reset(sender: &Sender<Message>, children: bool) -> Message {
    sender.send(Message::Reset(children)).expect(SENDER_ERR);
    create_success_message("Everything is being reset right now.")
}

/// Invoked when calling `pueue status`.
/// Return the current state.
fn get_status(state: &SharedState) -> Message {
    let state = state.lock().unwrap().clone();
    Message::StatusResponse(state)
}

/// Invoked when calling `pueue log`.
/// Return the current state and the stdou/stderr of all tasks to the client.
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
                match read_and_compress_log_files(*task_id, &state.settings.shared.pueue_directory)
                {
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

    if let Err(error) = state.save_settings() {
        return create_failure_message(format!("Failed while saving the config file: {}", error));
    }

    create_success_message(format!(
        "Parallel tasks setting for group {} adjusted",
        group
    ))
}

/// Initialize the shutdown procedure.
/// At first, the unix socket will be removed.
///
/// Next, the DaemonShutdown Message will be forwarded to the TaskHandler.
/// The TaskHandler then gracefully shuts down all child processes
/// and exits with std::proces::exit(0).
fn shutdown(sender: &Sender<Message>, state: &SharedState) -> Message {
    // Remove the unix socket
    {
        let state = state.lock().unwrap();
        if state.settings.shared.use_unix_socket {
            let path = &state.settings.shared.unix_socket_path;
            debug!("Check if a unit socket exists.");
            if std::path::PathBuf::from(&path).exists() {
                std::fs::remove_file(&path).expect("Failed to remove unix socket on shutdown");
            }
            debug!("Removed the unix socket.");
        }
    }

    // Notify the task handler
    sender.send(Message::DaemonShutdown).expect(SENDER_ERR);

    create_success_message("Daemon is shutting down")
}
