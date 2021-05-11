use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::PathBuf;
use std::process::Child;
use std::process::Stdio;
use std::sync::mpsc::Receiver;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::prelude::*;
use handlebars::Handlebars;
use log::{debug, error, info, warn};

use pueue_lib::log::*;
use pueue_lib::network::message::*;
use pueue_lib::state::{GroupStatus, SharedState};
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::platform::process_helper::*;

pub struct TaskHandler {
    state: SharedState,
    receiver: Receiver<Message>,
    children: BTreeMap<usize, Child>,
    callbacks: Vec<Child>,
    full_reset: bool,
    // Some static settings that are extracted from `state.settings` for convenience purposes.
    pueue_directory: PathBuf,
    callback: Option<String>,
}

/// Pueue directly interacts with processes.
/// Since these interactions can vary depending on the current platform, this enum is introduced.
/// The intend is to keep any platform specific code out of the top level code.
/// Even if that implicates adding some layer of abstraction.
#[derive(Debug)]
pub enum ProcessAction {
    Pause,
    Resume,
    Kill,
}

impl TaskHandler {
    pub fn new(state: SharedState, receiver: Receiver<Message>) -> Self {
        // Extract some static settings we often need.
        // This prevents locking the State all the time.
        let (pueue_directory, callback) = {
            let state = state.lock().unwrap();
            (
                state.settings.shared.pueue_directory(),
                state.settings.daemon.callback.clone(),
            )
        };

        TaskHandler {
            state,
            receiver,
            children: BTreeMap::new(),
            callbacks: Vec::new(),
            full_reset: false,
            pueue_directory,
            callback,
        }
    }
}

impl TaskHandler {
    /// Main loop of the task handler.
    /// In here a few things happen:
    ///
    /// - Receive and handle instructions from the client.
    /// - Handle finished tasks, i.e. cleanup processes, update statuses.
    /// - If the client requested a reset: reset the state if all children have been killed and handled.
    /// - Callback handling logic. This is rather uncritical.
    /// - Enqueue any stashed processes which are ready for being queued.
    /// - Ensure tasks with dependencies have no failed ancestors
    /// - Check whether we can spawn new tasks.
    pub fn run(&mut self) {
        loop {
            self.receive_commands();
            self.handle_finished_tasks();
            self.handle_reset();
            self.check_callbacks();
            self.enqueue_delayed_tasks();
            self.check_failed_dependencies();
            if !self.full_reset {
                self.check_new();
            }
        }
    }

    /// Search and return the next task that can be started.
    /// Precondition for a task to be started:
    /// - is in Queued state
    /// - There are free slots in the task's group
    /// - The group is running
    /// - has all its dependencies in `Done` state
    fn get_next_task_id(&mut self) -> Option<usize> {
        let state = self.state.lock().unwrap();
        // Check how many tasks are running in each group
        let mut running_tasks_per_group: HashMap<String, usize> = HashMap::new();

        // Create a default group for tasks without an explicit group
        running_tasks_per_group.insert("default".into(), 0);

        // Walk through all tasks and save the number of running tasks by group
        for (_, task) in state.tasks.iter() {
            // We are only interested in currently running tasks.
            if ![TaskStatus::Running, TaskStatus::Paused].contains(&task.status) {
                continue;
            }

            match running_tasks_per_group.get_mut(&task.group) {
                Some(count) => {
                    *count += 1;
                }
                None => {
                    running_tasks_per_group.insert(task.group.clone(), 1);
                }
            }
        }

        state
            .tasks
            .iter()
            .filter(|(_, task)| task.status == TaskStatus::Queued)
            .filter(|(_, task)| {
                // The task is assigned to a group.
                // First let's check if the group is running. If it isn't, simply return false.
                if !matches!(state.groups.get(&task.group), Some(GroupStatus::Running)) {
                    return false;
                }

                // If there's no running task for the group yet, we can safely return true
                //
                // If there are running tasks for this group, we have to ensure that there are
                // fewer running tasks than allowed for this group.
                match running_tasks_per_group.get(&task.group) {
                    None => true,
                    Some(count) => match state.settings.daemon.groups.get(&task.group) {
                        Some(allowed) => count < allowed,
                        None => {
                            error!(
                                "Got task with unknown group {}. Please report this!",
                                &task.group
                            );
                            false
                        }
                    },
                }
            })
            .find(|(_, task)| {
                // Check whether all dependencies for this task are fulfilled.
                task.dependencies
                    .iter()
                    .flat_map(|id| state.tasks.get(id))
                    .all(|task| task.status == TaskStatus::Done && !task.failed())
            })
            .map(|(id, _)| *id)
    }

    /// Users can issue to reset the daemon.
    /// If that's the case, the `self.full_reset` flag is set to true, all children are killed
    /// and no new tasks will be spawned.
    /// This function checks, if all killed children have been handled.
    /// If that's the case, completely reset the state
    fn handle_reset(&mut self) {
        // The daemon got a reset request and all children already finished
        if self.full_reset && self.children.is_empty() {
            let mut state = self.state.lock().unwrap();
            state.reset();
            state.set_status_for_all_groups(GroupStatus::Running);
            reset_task_log_directory(&self.pueue_directory);
            self.full_reset = false;
        }
    }

    /// See if we can start a new queued task.
    fn check_new(&mut self) {
        // Get the next task id that can be started
        if let Some(id) = self.get_next_task_id() {
            self.start_process(id);
        }
    }

    /// Ensure that no `Queued` tasks have any failed dependencies.
    /// Otherwise set their status to `Done` and result to `DependencyFailed`.
    fn check_failed_dependencies(&mut self) {
        // Clone the state ref, so we don't have two mutable borrows later on.
        let state_ref = self.state.clone();
        let mut state = state_ref.lock().unwrap();

        // Get id's of all tasks with failed dependencies
        let has_failed_deps: Vec<_> = state
            .tasks
            .iter()
            .filter(|(_, task)| task.status == TaskStatus::Queued && !task.dependencies.is_empty())
            .filter_map(|(id, task)| {
                // At this point we got all queued tasks with dependencies.
                // Go through all dependencies and ensure they didn't fail.
                let failed = task
                    .dependencies
                    .iter()
                    .flat_map(|id| state.tasks.get(id))
                    .filter(|task| task.failed())
                    .map(|task| task.id)
                    .next();

                failed.map(|f| (*id, f))
            })
            .collect();

        // Update the state of all tasks with failed dependencies.
        for (id, _) in has_failed_deps {
            // Get the task's group, since we have to check if it's paused.
            let group = if let Some(task) = state.tasks.get(&id) {
                task.group.clone()
            } else {
                continue;
            };

            // Only update the status, if the group isn't paused.
            // This allows users to fix and restart dependencies in-place without
            // breaking the dependency chain.
            if matches!(state.groups.get(&group).unwrap(), GroupStatus::Paused) {
                continue;
            }

            let task = state.tasks.get_mut(&id).unwrap();
            task.status = TaskStatus::Done;
            task.result = Some(TaskResult::DependencyFailed);
            task.start = Some(Local::now());
            task.end = Some(Local::now());
            self.spawn_callback(&task);
        }
    }

    /// Actually spawn a new sub process
    /// The output of subprocesses is piped into a seperate file for easier access
    fn start_process(&mut self, task_id: usize) {
        // Already get the mutex here to ensure that the state won't be manipulated
        // while we are looking for a task to start.
        // Also clone the state ref, so we don't have two mutable borrows later on.
        let state_ref = self.state.clone();
        let mut state = state_ref.lock().unwrap();

        // Check if the task exists and can actually be spawned. Otherwise do an early return.
        match state.tasks.get(&task_id) {
            Some(task) => {
                if !vec![TaskStatus::Stashed, TaskStatus::Queued, TaskStatus::Paused]
                    .contains(&task.status)
                {
                    info!("Tried to start task with status: {}", task.status);
                    return;
                }
            }
            None => {
                info!("Tried to start non-existing task: {}", task_id);
                return;
            }
        };

        // Try to get the log files to which the output of the process
        // will be written to. Error if this doesn't work!
        let (stdout_log, stderr_log) = match create_log_file_handles(task_id, &self.pueue_directory)
        {
            Ok((out, err)) => (out, err),
            Err(err) => {
                error!("Failed to create child log files: {:?}", err);
                return;
            }
        };

        // Get all necessary info for starting the task
        let (command, path, envs) = {
            let task = state.tasks.get(&task_id).unwrap();
            (task.command.clone(), task.path.clone(), task.envs.clone())
        };

        // Spawn the actual subprocess
        let mut command = compile_shell_command(&command);

        let spawned_command = command
            .current_dir(path)
            .stdin(Stdio::piped())
            .envs(envs)
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log))
            .spawn();

        // Check if the task managed to spawn
        let child = match spawned_command {
            Ok(child) => child,
            Err(err) => {
                let error = format!("Failed to spawn child {} with err: {:?}", task_id, err);
                error!("{}", error);
                clean_log_handles(task_id, &self.pueue_directory);

                // Update all necessary fields on the task.
                let group = {
                    let task = state.tasks.get_mut(&task_id).unwrap();
                    task.status = TaskStatus::Done;
                    task.result = Some(TaskResult::FailedToSpawn(error));
                    task.start = Some(Local::now());
                    task.end = Some(Local::now());
                    task.enqueue_at = None;
                    self.spawn_callback(&task);

                    task.group.clone()
                };

                state.handle_task_failure(group);
                state.save();
                return;
            }
        };
        self.children.insert(task_id, child);

        let task = state.tasks.get_mut(&task_id).unwrap();

        task.start = Some(Local::now());
        task.status = TaskStatus::Running;
        task.enqueue_at = None;

        info!("Started task: {}", task.command);
        state.save();
    }

    /// As time passes, some delayed tasks may need to be enqueued.
    /// Gather all stashed tasks and enqueue them if it is after the task's enqueue_at
    fn enqueue_delayed_tasks(&mut self) {
        let mut state = self.state.lock().unwrap();

        let mut changed = false;
        for (_, task) in state.tasks.iter_mut() {
            if task.status != TaskStatus::Stashed {
                continue;
            }

            if let Some(time) = task.enqueue_at {
                if time <= Local::now() {
                    info!("Enqueuing delayed task : {}", task.id);

                    task.status = TaskStatus::Queued;
                    task.enqueue_at = None;
                    changed = true;
                }
            }
        }
        // Save the state if a task has been enqueued
        if changed {
            state.save();
        }
    }

    /// Check whether there are any finished processes
    /// In case there are, handle them and update the shared state
    fn handle_finished_tasks(&mut self) {
        let finished = self.get_finished();

        // Nothing to do. Early return
        if finished.is_empty() {
            return;
        }

        // Clone the state ref, so we don't have two mutable borrows later on.
        let state_ref = self.state.clone();
        let mut state = state_ref.lock().unwrap();

        for (task_id, error) in finished.iter() {
            // Handle std::io errors on child processes.
            // I have never seen something like this, but it might happen.
            if let Some(error) = error {
                let _child = self
                    .children
                    .remove(task_id)
                    .expect("Errored child went missing while handling finished task.");

                let group = {
                    let mut task = state.tasks.get_mut(&task_id).unwrap();
                    task.status = TaskStatus::Done;
                    task.end = Some(Local::now());
                    task.result = Some(TaskResult::Errored);
                    self.spawn_callback(&task);

                    task.group.clone()
                };
                error!("Child {} failed with io::Error: {:?}", task_id, error);

                state.handle_task_failure(group);
                continue;
            }

            // Handle any tasks that exited with some kind of exit code
            let mut child = self
                .children
                .remove(task_id)
                .expect("Child of task {} went away while handling finished task.");

            // Get the exit code of the child.
            // Errors really shouldn't happen in here, since we already checked if it's finished
            // with try_wait() before.
            let exit_code_result = child.wait();
            let exit_code = exit_code_result
                .context(format!(
                    "Failed on wait() for finished task {} with error: {:?}",
                    task_id, error
                ))
                .unwrap()
                .code();

            // Processes with exit code 0 exited successfully
            // Processes with `None` have been killed by a Signal
            let result = match exit_code {
                Some(0) => Some(TaskResult::Success),
                Some(exit_code) => Some(TaskResult::Failed(exit_code)),
                None => Some(TaskResult::Killed),
            };

            // Update all properties on the task and get the group for later
            let group = {
                let mut task = state
                    .tasks
                    .get_mut(&task_id)
                    .expect("Task was removed before child process has finished!");

                task.status = TaskStatus::Done;
                task.end = Some(Local::now());
                task.result = result.clone();
                self.spawn_callback(&task);

                task.group.clone()
            };

            if let Some(TaskResult::Failed(_)) = result {
                state.handle_task_failure(group);
            }

            // Already remove the output files, if the daemon is being reset anyway
            if self.full_reset {
                clean_log_handles(*task_id, &self.pueue_directory);
            }
        }

        state.save()
    }

    /// Gather all finished tasks and sort them by finished and errored.
    /// Returns a list of finished task ids and whether they errored or not.
    fn get_finished(&mut self) -> Vec<(usize, Option<std::io::Error>)> {
        let mut finished = Vec::new();
        for (id, child) in self.children.iter_mut() {
            match child.try_wait() {
                // Handle a child error.
                Err(error) => {
                    finished.push((*id, Some(error)));
                }
                // Child process did not exit yet
                Ok(None) => continue,
                Ok(_exit_status) => {
                    info!("Task {} just finished", id);
                    finished.push((*id, None));
                }
            }
        }

        finished
    }

    /// Some client instructions require immediate action by the task handler
    /// This function is also responsible for waiting
    fn receive_commands(&mut self) {
        // Sleep for a few milliseconds. We don't want to hurt the CPU.
        let timeout = Duration::from_millis(200);
        // Don't use recv_timeout for now, until this bug get's fixed.
        // https://github.com/rust-lang/rust/issues/39364
        //match self.receiver.recv_timeout(timeout) {
        sleep(timeout);

        if let Ok(message) = self.receiver.try_recv() {
            self.handle_message(message);
        };
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::Pause(message) => self.pause(message),
            Message::Start(message) => self.start(message),
            Message::Kill(message) => self.kill(message),
            Message::Send(message) => self.send(message),
            Message::Reset(message) => self.reset(message),
            Message::DaemonShutdown => self.shutdown(),
            _ => info!("Received unhandled message {:?}", message),
        }
    }

    /// This is a small wrapper around the real platform dependant process handling logic
    /// It only ensures, that the process we want to manipulate really does exists.
    fn perform_action(&mut self, id: usize, action: ProcessAction, children: bool) -> Result<bool> {
        match self.children.get(&id) {
            Some(child) => {
                debug!("Executing action {:?} to {}", action, id);
                send_signal_to_child(child, &action, children)?;

                Ok(true)
            }
            None => {
                error!(
                    "Tried to execute action {:?} to non existing task {}",
                    action, id
                );
                Ok(false)
            }
        }
    }

    /// Handle the start message:
    /// 1. Either start the daemon and all tasks.
    /// 2. Or force the start of specific tasks.
    fn start(&mut self, message: StartMessage) {
        // Only start specific tasks
        // This is handled separately, since this can also force-spawn processes
        if !message.task_ids.is_empty() {
            for id in &message.task_ids {
                // Continue all children that are simply paused
                if self.children.contains_key(id) {
                    self.continue_task(*id, message.children);
                } else {
                    // Start processes for all tasks that haven't been started yet
                    self.start_process(*id);
                }
            }
            return;
        }

        // Get the keys of all tasks that should be resumed
        // These can either be
        // - All running tasks
        // - The paused tasks of a specific group
        // - The paused tasks of the default queue
        let keys: Vec<usize> = if message.all {
            // Resume all groups and the default queue
            info!("Resuming everything");
            let mut state = self.state.lock().unwrap();
            state.set_status_for_all_groups(GroupStatus::Running);

            self.children.keys().cloned().collect()
        } else {
            let mut state = self.state.lock().unwrap();
            // Ensure that a given group exists. (Might not happen due to concurrency)
            if !state.groups.contains_key(&message.group) {
                return;
            }
            // Set the group to running.
            state
                .groups
                .insert(message.group.clone(), GroupStatus::Running);
            info!("Resuming group {}", &message.group);

            state.task_ids_in_group_with_stati(&message.group, vec![TaskStatus::Paused])
        };

        // Resume all specified paused tasks
        for id in keys {
            self.continue_task(id, message.children);
        }
    }

    /// Send a start signal to a paused task to continue execution.
    fn continue_task(&mut self, id: usize, children: bool) {
        if !self.children.contains_key(&id) {
            return;
        }
        {
            // Task is already done
            let state = self.state.lock().unwrap();
            if state.tasks.get(&id).unwrap().is_done() {
                return;
            }
        }
        match self.perform_action(id, ProcessAction::Resume, children) {
            Err(err) => warn!("Failed resuming task {}: {:?}", id, err),
            Ok(success) => {
                if success {
                    let mut state = self.state.lock().unwrap();
                    state.change_status(id, TaskStatus::Running);
                }
            }
        }
    }

    /// Handle the pause message:
    /// 1. Either pause the daemon and all tasks.
    /// 2. Or only pause specific tasks.
    fn pause(&mut self, message: PauseMessage) {
        // Get the keys of all tasks that should be resumed
        // These can either be
        // - Specific tasks
        // - All running tasks
        // - The paused tasks of a group
        // - The paused tasks of the default queue
        // Only pause specific tasks
        let keys: Vec<usize> = if !message.task_ids.is_empty() {
            message.task_ids
        } else if message.all {
            // Pause all running tasks
            let mut state = self.state.lock().unwrap();
            state.set_status_for_all_groups(GroupStatus::Paused);

            info!("Pausing everything");
            self.children.keys().cloned().collect()
        } else {
            // Ensure that a given group exists. (Might not happen due to concurrency)
            let mut state = self.state.lock().unwrap();
            if !state.groups.contains_key(&message.group) {
                return;
            }
            // Pause a specific group.
            state
                .groups
                .insert(message.group.clone(), GroupStatus::Paused);
            info!("Pausing group {}", &message.group);

            state.task_ids_in_group_with_stati(&message.group, vec![TaskStatus::Running])
        };

        // Pause all specified tasks
        if !message.wait {
            for id in keys {
                self.pause_task(id, message.children);
            }
        }
    }

    /// Pause a specific task.
    /// Send a signal to the process to actually pause the OS process.
    fn pause_task(&mut self, id: usize, children: bool) {
        match self.perform_action(id, ProcessAction::Pause, children) {
            Err(err) => error!("Failed pausing task {}: {:?}", id, err),
            Ok(success) => {
                if success {
                    let mut state = self.state.lock().unwrap();
                    state.change_status(id, TaskStatus::Paused);
                }
            }
        }
    }

    /// Handle the kill message:
    /// 1. Kill specific tasks.
    /// 2. Kill all tasks.
    /// 3. Kill all tasks of a specific group.
    /// 4. Kill all tasks of the default queue.
    fn kill(&mut self, message: KillMessage) {
        // Get the keys of all tasks that should be resumed
        // These can either be
        // - Specific tasks
        // - All running tasks
        // - The paused tasks of a group
        // - The paused tasks of the default queue
        // Only pause specific tasks
        let task_ids: Vec<usize> = if !message.task_ids.is_empty() {
            message.task_ids
        } else if message.all {
            // Pause all running tasks
            let mut state = self.state.lock().unwrap();
            state.set_status_for_all_groups(GroupStatus::Paused);

            info!("Killing all running tasks");
            self.children.keys().cloned().collect()
        } else {
            // Ensure that a given group exists. (Might not happen due to concurrency)
            let mut state = self.state.lock().unwrap();
            if !state.groups.contains_key(&message.group) {
                return;
            }
            // Pause a specific group.
            state
                .groups
                .insert(message.group.clone(), GroupStatus::Paused);
            info!("Killing tasks of group {}", &message.group);

            state.task_ids_in_group_with_stati(
                &message.group,
                vec![TaskStatus::Running, TaskStatus::Paused],
            )
        };

        for task_id in task_ids {
            self.kill_task(task_id, message.children);
        }
    }

    /// Kill a specific task and handle it accordingly.
    /// Triggered on `reset` and `kill`.
    fn kill_task(&mut self, task_id: usize, kill_children: bool) {
        if let Some(mut child) = self.children.get_mut(&task_id) {
            kill_child(task_id, &mut child, kill_children);
        } else {
            warn!("Tried to kill non-existing child: {}", task_id);
        }
    }

    /// Send some input to a child process.
    fn send(&mut self, message: SendMessage) {
        let task_id = message.task_id;
        let input = message.input;
        let child = match self.children.get_mut(&task_id) {
            Some(child) => child,
            None => {
                warn!(
                    "Task {} finished before input could be sent: {}",
                    task_id, input
                );
                return;
            }
        };
        {
            let child_stdin = child.stdin.as_mut().unwrap();
            if let Err(err) = child_stdin.write_all(&input.clone().into_bytes()) {
                warn!(
                    "Failed to send input to task {} with err {:?}: {}",
                    task_id, err, input
                );
            };
        }
    }

    /// Kill all children by using the `kill` function.
    /// Set the respective group's statuses to `Reset`. This will prevent new tasks from being spawned.
    fn reset(&mut self, message: ResetMessage) {
        {
            let mut state = self.state.lock().unwrap();
            state.set_status_for_all_groups(GroupStatus::Paused);
        }
        self.full_reset = true;

        let message = KillMessage {
            all: true,
            children: message.children,
            ..Default::default()
        };

        self.kill(message);
    }

    /// Users can specify a callback that's fired whenever a task finishes.
    /// Execute the callback by spawning a new subprocess.
    fn spawn_callback(&mut self, task: &Task) {
        // Return early, if there's no callback specified
        let callback = if let Some(callback) = &self.callback {
            callback
        } else {
            return;
        };

        // Build the callback command from the given template.
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        // Build templating variables.
        let mut parameters = HashMap::new();
        parameters.insert("id", task.id.to_string());
        parameters.insert("command", task.command.clone());
        parameters.insert("path", task.path.clone());
        parameters.insert("result", task.result.clone().unwrap().to_string());

        let print_time = |time: Option<DateTime<Local>>| {
            time.map(|time| time.timestamp().to_string())
                .unwrap_or_else(String::new)
        };
        parameters.insert("enqueue", print_time(task.enqueue_at));
        parameters.insert("start", print_time(task.start));
        parameters.insert("end", print_time(task.end));
        parameters.insert("group", task.group.clone());

        // Read the last 10 lines of output and make it available.
        if let Ok((stdout, stderr)) = read_last_log_file_lines(task.id, &self.pueue_directory, 10) {
            parameters.insert("stdout", stdout);
            parameters.insert("stderr", stderr);
        } else {
            parameters.insert("stdout", "".to_string());
            parameters.insert("stderr", "".to_string());
        }

        if let Some(TaskResult::Success) = &task.result {
            parameters.insert("exit_code", "0".into());
        } else if let Some(TaskResult::Failed(code)) = &task.result {
            parameters.insert("exit_code", code.to_string());
        } else {
            parameters.insert("exit_code", "None".into());
        }

        let callback_command = match handlebars.render_template(&callback, &parameters) {
            Ok(callback_command) => callback_command,
            Err(err) => {
                error!(
                    "Failed to create callback command from template with error: {}",
                    err
                );
                return;
            }
        };

        let mut command = compile_shell_command(&callback_command);

        // Spawn the callback subprocess and log if it fails.
        let spawn_result = command.spawn();
        let child = match spawn_result {
            Err(error) => {
                error!("Failed to spawn callback with error: {}", error);
                return;
            }
            Ok(child) => child,
        };

        debug!("Spawned callback for task {}", task.id);
        self.callbacks.push(child);
    }

    /// Look at all running callbacks and log any errors.
    /// If everything went smoothly, simply remove them from the list.
    fn check_callbacks(&mut self) {
        let mut finished = Vec::new();
        for (id, child) in self.callbacks.iter_mut().enumerate() {
            match child.try_wait() {
                // Handle a child error.
                Err(error) => {
                    error!("Callback failed with error {:?}", error);
                    finished.push(id);
                }
                // Child process did not exit yet.
                Ok(None) => continue,
                Ok(exit_status) => {
                    info!("Callback finished with exit code {:?}", exit_status);
                    finished.push(id);
                }
            }
        }

        finished.reverse();
        for id in finished.iter() {
            self.callbacks.remove(*id);
        }
    }

    /// Gracefully shutdown the task handler.
    /// This includes killing all child processes
    ///
    /// Afterwards we can actually exit the program
    fn shutdown(&mut self) {
        info!("Killing all children due to shutdown.");

        let task_ids: Vec<usize> = self.children.keys().cloned().collect();
        for task_id in task_ids {
            let child = self.children.remove(&task_id);

            if let Some(mut child) = child {
                info!("Killing child {}", &task_id);
                kill_child(task_id, &mut child, true);
            } else {
                error!("Fail to get child {} for killing", &task_id);
            }
        }

        // Exit pueued
        std::process::exit(0)
    }
}
