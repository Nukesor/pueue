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
        // Check how many tasks are running in each group
        let mut running_tasks_per_group: HashMap<String, usize> = HashMap::new();

        // Create a default group for tasks without an explicit group
        running_tasks_per_group.insert("default".into(), 0);

        // Walk through all tasks and save the number of running tasks by group
        for (_, task) in state.tasks.iter() {
            // We are only interested in currently running tasks.
            if !matches!(task.status, TaskStatus::Running | TaskStatus::Paused) {
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
                    task.status = TaskStatus::Done(TaskResult::FailedToSpawn(error));
                    task.start = Some(Local::now());
                    task.end = Some(Local::now());
                    self.spawn_callback(task);

                    task.group.clone()
                };

                pause_on_failure(state, group);
                ok_or_shutdown!(self, save_state(state));
                return;
            }
        };
        self.children.insert(task_id, child);

        let task = state.tasks.get_mut(&task_id).unwrap();

        task.start = Some(Local::now());
        task.status = TaskStatus::Running;

        info!("Started task: {}", task.command);
        ok_or_shutdown!(self, save_state(state));
    }
}
