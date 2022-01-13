use super::*;

use crate::ok_or_shutdown;
use crate::state_helper::{pause_on_failure, save_state, LockedState};

impl TaskHandler {
    /// See if we can start a new queued task.
    pub fn spawn_new(&mut self) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();
        // Check whether a new task can be started.
        // Spawn tasks until we no longer have free slots available.
        while let Some(id) = self.get_next_task_id(&state) {
            self.start_process(id, &mut state);
        }
    }

    /// Search and return the next task that can be started.
    /// Precondition for a task to be started:
    /// - is in Queued state
    /// - There are free slots in the task's group
    /// - The group is running
    /// - has all its dependencies in `Done` state
    pub fn get_next_task_id(&mut self, state: &LockedState) -> Option<usize> {
        state
            .tasks
            .iter()
            .filter(|(_, task)| task.status == TaskStatus::Queued)
            .filter(|(_, task)| {
                // Make sure the task is assigned to an existing group.
                let group = match state.groups.get(&task.group) {
                    Some(group) => group,
                    None => {
                        error!(
                            "Got task with unknown group {}. Please report this!",
                            &task.group
                        );
                        return false;
                    }
                };

                // Let's check if the group is running. If it isn't, simply return false.
                if group.status != GroupStatus::Running {
                    return false;
                }

                // Get the currently running tasks by looking at the actually running processes.
                // They're sorted by group, which makes this quite convenient.
                let running_tasks = match self.children.0.get(&task.group) {
                    Some(children) => children.len(),
                    None => {
                        error!(
                            "Got valid group {}, but no worker pool has been initialized. This is a bug!",
                            &task.group
                        );
                        return false
                    }
                };

                // Make sure there are free slots in the task's group
                running_tasks < group.parallel_tasks
            })
            .find(|(_, task)| {
                // Check whether all dependencies for this task are fulfilled.
                task.dependencies
                    .iter()
                    .flat_map(|id| state.tasks.get(id))
                    .all(|task| matches!(task.status, TaskStatus::Done(TaskResult::Success)))
            })
            .map(|(id, _)| *id)
    }

    /// Actually spawn a new sub process
    /// The output of subprocesses is piped into a seperate file for easier access
    pub fn start_process(&mut self, task_id: usize, state: &mut LockedState) {
        // Check if the task exists and can actually be spawned. Otherwise do an early return.
        match state.tasks.get(&task_id) {
            Some(task) => {
                if !matches!(
                    &task.status,
                    TaskStatus::Stashed { .. } | TaskStatus::Queued | TaskStatus::Paused
                ) {
                    info!("Tried to start task with status: {}", task.status);
                    return;
                }
            }
            None => {
                info!("Tried to start non-existing task: {task_id}");
                return;
            }
        };

        // Try to get the log files to which the output of the process will be written to.
        // Panic if this doesn't work! This is unrecoverable.
        let (stdout_log, stderr_log) = match create_log_file_handles(task_id, &self.pueue_directory)
        {
            Ok((out, err)) => (out, err),
            Err(err) => {
                panic!("Failed to create child log files: {err:?}");
            }
        };

        // Get all necessary info for starting the task
        let (command, path, group, mut envs) = {
            let task = state.tasks.get(&task_id).unwrap();
            (
                task.command.clone(),
                task.path.clone(),
                task.group.clone(),
                task.envs.clone(),
            )
        };

        // Build the shell command that should be executed.
        let mut command = compile_shell_command(&command);

        // Determine the worker's id depending on the current group.
        // Inject that info into the environment.
        let worker_id = self.children.get_next_group_worker(&group);
        envs.insert("PUEUE_GROUP".into(), group.clone());
        envs.insert("PUEUE_WORKER_ID".into(), worker_id.to_string());

        // Spawn the actual subprocess
        let spawned_command = command
            .current_dir(path)
            .stdin(Stdio::piped())
            .envs(envs.clone())
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log))
            .spawn();

        // Check if the task managed to spawn
        let child = match spawned_command {
            Ok(child) => child,
            Err(err) => {
                let error = format!("Failed to spawn child {task_id} with err: {err:?}");
                error!("{}", error);
                clean_log_handles(task_id, &self.pueue_directory);

                // Update all necessary fields on the task.
                let group = {
                    let task = state.tasks.get_mut(&task_id).unwrap();
                    task.status = TaskStatus::Done(TaskResult::FailedToSpawn(error));
                    task.start = Some(Local::now());
                    task.end = Some(Local::now());
                    self.spawn_callback(task);

                    task.group.clone()
                };

                pause_on_failure(state, &group);
                ok_or_shutdown!(self, save_state(state));
                return;
            }
        };

        // Save the process handle in our self.children datastructure.
        self.children.add_child(&group, worker_id, task_id, child);

        let task = state.tasks.get_mut(&task_id).unwrap();
        task.start = Some(Local::now());
        task.status = TaskStatus::Running;
        // Overwrite the task's environment variables with the new ones, containing the
        // PUEUE_WORKER_ID and PUEUE_GROUP variables.
        task.envs = envs;

        info!("Started task: {}", task.command);
        ok_or_shutdown!(self, save_state(state));
    }
}
