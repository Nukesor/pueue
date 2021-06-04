use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use chrono::prelude::*;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};

use crate::error::Error;
use crate::settings::Settings;
use crate::task::{Task, TaskResult, TaskStatus};

pub type SharedState = Arc<Mutex<State>>;

/// Represents the current status of a group.
/// Each group acts as a queue and can be managed individually.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GroupStatus {
    Running,
    Paused,
}

/// This is the full representation of the current state of the Pueue daemon.
///
/// This includes
/// - The currently used settings.
/// - The full task list
/// - The current status of all tasks
/// - All known groups.
///
/// However, the State does NOT include:
/// - Information about child processes
/// - Handles to child processes
///
/// That information is saved in the daemon's TaskHandler.
///
/// Most functions implemented on the state shouldn't be used by third party software.
/// The daemon is constantly changing and persisting the state. \
/// Any changes applied to a state and saved to disk, will most likely be overwritten
/// after a short time.
///
///
/// The daemon uses the state as a piece of shared memory between it's threads.
/// It's wrapped in a MutexGuard, which allows us to guarantee sequential access to any crucial
/// information, such as status changes and incoming commands by the client.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    /// The current settings used by the daemon.
    pub settings: Settings,
    /// All tasks currently managed by the daemon.
    pub tasks: BTreeMap<usize, Task>,
    /// All groups
    pub groups: BTreeMap<String, GroupStatus>,
    config_path: Option<PathBuf>,
}

impl State {
    /// Create a new default state.
    pub fn new(settings: &Settings, config_path: Option<PathBuf>) -> State {
        // Create a default group state.
        let mut groups = BTreeMap::new();
        for group in settings.daemon.groups.keys() {
            groups.insert(group.into(), GroupStatus::Running);
        }

        let mut state = State {
            settings: settings.clone(),
            tasks: BTreeMap::new(),
            groups,
            config_path,
        };
        state.create_group("default");
        state
    }

    /// Add a new task
    pub fn add_task(&mut self, mut task: Task) -> usize {
        let next_id = match self.tasks.keys().max() {
            None => 0,
            Some(id) => id + 1,
        };
        task.id = next_id;
        self.tasks.insert(next_id, task);

        next_id
    }

    /// A small helper to change the status of a specific task.
    pub fn change_status(&mut self, id: usize, new_status: TaskStatus) {
        if let Some(ref mut task) = self.tasks.get_mut(&id) {
            task.status = new_status;
        };
    }

    /// Set the time a specific task should be enqueued at.
    pub fn set_enqueue_at(&mut self, id: usize, enqueue_at: Option<DateTime<Local>>) {
        if let Some(ref mut task) = self.tasks.get_mut(&id) {
            task.enqueue_at = enqueue_at;
        }
    }

    /// Add a new group to the daemon. \
    /// This also check if the given group already exists.
    /// Create a state.group entry and a settings.group entry, if it doesn't.
    pub fn create_group(&mut self, group: &str) {
        if self.settings.daemon.groups.get(group).is_none() {
            self.settings.daemon.groups.insert(group.into(), 1);
        }
        if self.groups.get(group).is_none() {
            self.groups.insert(group.into(), GroupStatus::Running);
        }
    }

    /// Remove a group.
    /// This also iterates through all tasks and sets any tasks' group
    /// to the `default` group if it matches the deleted group.
    pub fn remove_group(&mut self, group: &str) -> Result<(), Error> {
        if group.eq("default") {
            return Err(Error::Generic(
                "You cannot remove the default group.".into(),
            ));
        }

        self.settings.daemon.groups.remove(group);
        self.groups.remove(group);

        // Reset all tasks with removed group to the default.
        for (_, task) in self.tasks.iter_mut() {
            if task.group.eq(group) {
                task.set_default_group();
            }
        }

        self.save()?;
        self.save_settings()
    }

    /// Set the group status (running/paused) for all groups including the default queue.
    pub fn set_status_for_all_groups(&mut self, status: GroupStatus) {
        let keys = self.groups.keys().cloned().collect::<Vec<String>>();
        for key in keys {
            self.groups.insert(key, status.clone());
        }
    }

    /// Get all ids of task with a specific state inside a specific group.
    pub fn task_ids_in_group_with_stati(&self, group: &str, stati: Vec<TaskStatus>) -> Vec<usize> {
        self.tasks
            .iter()
            .filter(|(_, task)| stati.contains(&task.status))
            .filter(|(_, task)| task.group.eq(group))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get all ids of task inside a specific group.
    pub fn task_ids_in_group(&self, group: &str) -> Vec<usize> {
        self.tasks
            .iter()
            .filter(|(_, task)| task.group.eq(group))
            .map(|(id, _)| *id)
            .collect()
    }

    /// This checks, whether some tasks have one of the specified statuses. \
    /// The first result is the list of task_ids that match these statuses. \
    /// The second result is the list of task_ids that don't match these statuses. \
    ///
    /// By default, this checks all tasks in the current state. If a list of task_ids is
    /// provided as the third parameter, only those tasks will be checked.
    pub fn tasks_in_statuses(
        &self,
        statuses: Vec<TaskStatus>,
        task_ids: Option<Vec<usize>>,
    ) -> (Vec<usize>, Vec<usize>) {
        let task_ids = match task_ids {
            Some(ids) => ids,
            None => self.tasks.keys().cloned().collect(),
        };

        self.task_ids_in_statuses(task_ids, statuses)
    }

    /// Same as [tasks_in_statuses], but only checks for tasks of a specific group.
    pub fn tasks_of_group_in_statuses(
        &self,
        statuses: Vec<TaskStatus>,
        group: &str,
    ) -> (Vec<usize>, Vec<usize>) {
        // Return empty vectors, if there's no such group.
        if !self.groups.contains_key(group) {
            return (vec![], vec![]);
        }

        // Filter all task ids of tasks that match the given group.
        let task_ids = self
            .tasks
            .iter()
            .filter(|(_, task)| task.group == group)
            .map(|(id, _)| *id)
            .collect();

        self.task_ids_in_statuses(task_ids, statuses)
    }

    /// Internal function used to check which of the given tasks are in one of the given statuses.
    ///
    /// Returns a tuple of all (matching_task_ids, non_matching_task_ids).
    fn task_ids_in_statuses(
        &self,
        task_ids: Vec<usize>,
        statuses: Vec<TaskStatus>,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut matching = Vec::new();
        let mut mismatching = Vec::new();

        // Filter all task id's that match the provided statuses.
        for task_id in task_ids.iter() {
            // Check whether the task exists and save all non-existing task ids.
            match self.tasks.get(&task_id) {
                None => {
                    mismatching.push(*task_id);
                    continue;
                }
                Some(task) => {
                    // Check whether the task status matches the specified statuses.
                    if statuses.contains(&task.status) {
                        matching.push(*task_id);
                    } else {
                        mismatching.push(*task_id);
                    }
                }
            };
        }

        (matching, mismatching)
    }

    /// Check if a task can be deleted. \
    /// We have to check all dependant tasks, that haven't finished yet.
    /// This is necessary to prevent deletion of tasks which are specified as a dependency.
    ///
    /// `to_delete` A list of task ids, which should also be deleted.
    ///             This allows to remove dependency tasks as well as their dependants.
    pub fn is_task_removable(&self, task_id: &usize, to_delete: &[usize]) -> bool {
        // Get all task ids of any dependant tasks.
        let dependants: Vec<usize> = self
            .tasks
            .iter()
            .filter(|(_, task)| {
                task.dependencies.contains(&task_id) && task.status != TaskStatus::Done
            })
            .map(|(_, task)| task.id)
            .collect();

        if dependants.is_empty() {
            return true;
        }

        // Check if the dependants are supposed to be deleted as well.
        let should_delete_dependants = dependants.iter().all(|task_id| to_delete.contains(task_id));
        if !should_delete_dependants {
            return false;
        }

        // Lastly, do a recursive check if there are any dependants on our dependants
        dependants
            .iter()
            .all(|task_id| self.is_task_removable(task_id, to_delete))
    }

    /// A small helper for handling task failures. \
    /// Users can specify whether they want to pause the task's group or the
    /// whole daemon on a failed tasks. This function wraps that logic and decides if anything should be
    /// paused depending on the current settings.
    ///
    /// `group` should be the name of the failed task.
    pub fn handle_task_failure(&mut self, group: String) {
        if self.settings.daemon.pause_group_on_failure {
            self.groups.insert(group, GroupStatus::Paused);
        } else if self.settings.daemon.pause_all_on_failure {
            self.set_status_for_all_groups(GroupStatus::Paused);
        }
    }

    /// Do a full reset of the state.
    /// This doesn't reset any processes!
    pub fn reset(&mut self) -> Result<(), Error> {
        self.backup()?;
        self.tasks = BTreeMap::new();
        self.set_status_for_all_groups(GroupStatus::Running);

        self.save()
    }

    /// A small convenience wrapper for saving the settings to a file.
    pub fn save_settings(&self) -> Result<(), Error> {
        self.settings.save(&self.config_path)
    }

    /// Convenience wrapper around save_to_file.
    pub fn save(&self) -> Result<(), Error> {
        self.save_to_file(false)
    }

    /// Save the current current state in a file with a timestamp.
    /// At the same time remove old state logs from the log directory.
    /// This function is called, when large changes to the state are applied, e.g. clean/reset.
    pub fn backup(&self) -> Result<(), Error> {
        self.save_to_file(true)?;
        self.rotate()
            .map_err(|err| Error::Generic(format!("Failed to rotate old log files:\n{}", err)))?;
        Ok(())
    }

    /// Save the current state to disk. \
    /// We do this to restore in case of a crash. \
    /// If log == true, the file will be saved with a time stamp.
    ///
    /// In comparison to the daemon -> client communication, the state is saved
    /// as JSON for better readability and debug purposes.
    fn save_to_file(&self, log: bool) -> Result<(), Error> {
        let serialized = serde_json::to_string(&self);
        if let Err(error) = serialized {
            return Err(Error::StateSave(format!(
                "Failed to serialize state:\n\n{}",
                error
            )));
        }

        let serialized = serialized.unwrap();
        let path = self.settings.shared.pueue_directory();
        let (temp, real) = if log {
            let path = path.join("log");
            let now: DateTime<Utc> = Utc::now();
            let time = now.format("%Y-%m-%d_%H-%M-%S");
            (
                path.join(format!("{}_state.json.partial", time)),
                path.join(format!("{}_state.json", time)),
            )
        } else {
            (path.join("state.json.partial"), path.join("state.json"))
        };

        // Write to temporary log file first, to prevent loss due to crashes.
        fs::write(&temp, serialized)
            .map_err(|err| Error::StateSave(format!("Failed to write file:\n\n{}", err)))?;

        // Overwrite the original with the temp file, if everything went fine.
        fs::rename(&temp, &real).map_err(|err| {
            Error::StateSave(format!("Failed to overwrite old log file:\n\n{}", err))
        })?;

        if log {
            debug!("State backup created at: {:?}", real);
        } else {
            debug!("State saved at: {:?}", real);
        }

        Ok(())
    }

    /// Restore the last state from a previous session. \
    /// The state is stored as json in the log directory.
    pub fn restore(&mut self) -> Result<(), Error> {
        let path = Path::new(&self.settings.shared.pueue_directory()).join("state.json");

        // Ignore if the file doesn't exist. It doesn't have to.
        if !path.exists() {
            info!(
                "Couldn't find state from previous session at location: {:?}",
                path
            );
            return Ok(());
        }
        info!("Start restoring state");

        // Try to load the file.
        let data = fs::read_to_string(&path)
            .map_err(|err| Error::StateSave(format!("Failed to read file:\n\n{}", err)))?;

        // Try to deserialize the state file.
        let mut state: State = serde_json::from_str(&data)
            .map_err(|err| Error::StateDeserialization(err.to_string()))?;

        // Copy group statuses from the previous state.
        for (group, _) in state.settings.daemon.groups {
            if let Some(status) = state.groups.get(&group) {
                self.groups.insert(group.clone(), status.clone());
            }
        }

        // Restore all tasks.
        // While restoring the tasks, check for any invalid/broken stati.
        for (task_id, task) in state.tasks.iter_mut() {
            // Handle ungraceful shutdowns while executing tasks.
            if task.status == TaskStatus::Running || task.status == TaskStatus::Paused {
                info!(
                    "Setting task {} with previous status {:?} to new status {:?}",
                    task.id,
                    task.status,
                    TaskResult::Killed
                );
                task.status = TaskStatus::Done;
                task.result = Some(TaskResult::Killed);
            }

            // Handle crash during editing of the task command.
            if task.status == TaskStatus::Locked {
                task.status = TaskStatus::Stashed;
            }

            // Go trough all tasks and set all groups that are no longer
            // listed in the configuration file to the default.
            if !self.settings.daemon.groups.contains_key(&task.group) {
                task.set_default_group();
            }

            // If there are any queued tasks, pause the group.
            // This should prevent any unwanted execution of tasks due to a system crash.
            if task.status == TaskStatus::Queued {
                info!(
                    "Pausing group {} to prevent unwanted execution of previous tasks",
                    &task.group
                );
                self.groups.insert(task.group.clone(), GroupStatus::Paused);
            }

            self.tasks.insert(*task_id, task.clone());
        }

        Ok(())
    }

    /// Remove old logs that aren't needed any longer.
    fn rotate(&self) -> Result<(), Error> {
        let path = self.settings.shared.pueue_directory().join("log");

        // Get all log files in the directory with their respective system time.
        let mut entries: BTreeMap<SystemTime, PathBuf> = BTreeMap::new();
        let mut directory_list = fs::read_dir(path)?;
        while let Some(Ok(entry)) = directory_list.next() {
            let path = entry.path();

            let metadata = entry.metadata()?;
            let time = metadata.modified()?;
            entries.insert(time, path);
        }

        // Remove all files above the threshold.
        // Old files are removed first (implictly by the BTree order).
        let mut number_entries = entries.len();
        let mut iter = entries.iter();
        while number_entries > 10 {
            if let Some((_, path)) = iter.next() {
                fs::remove_file(path)?;
                number_entries -= 1;
            }
        }

        Ok(())
    }
}
