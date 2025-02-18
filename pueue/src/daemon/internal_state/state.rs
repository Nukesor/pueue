use std::{
    collections::BTreeMap,
    fs::{read_to_string, File},
    io::{Read, Write},
    process::Child,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::Local;
use flate2::Compression;
use pueue_lib::{
    error::Error,
    network::message::request::Shutdown,
    state::{FilteredTasks, PUEUE_DEFAULT_GROUP},
    task::{Task, TaskStatus},
    Group, GroupStatus, Settings, State, TaskResult,
};
use serde::{Deserialize, Serialize};

use crate::{daemon::internal_state::children::Children, internal_prelude::*};

pub type SharedState = Arc<Mutex<InternalState>>;
pub type LockedState<'a> = MutexGuard<'a, InternalState>;

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
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct InternalState {
    pub inner: State,

    /// Whether we're currently in the process of a graceful shutdown.
    /// Depending on the shutdown type, we're exiting with different exitcodes.
    /// This is runtime state and won't be serialised to disk.
    #[serde(default, skip)]
    pub shutdown: Option<Shutdown>,

    /// Pueue's subprocess and worker pool representation.
    /// Take a look at [Children] for more info.
    /// This is runtime state and won't be serialised to disk.
    #[serde(default, skip)]
    pub children: Children,
    /// These are the currently running callbacks. They're usually very short-lived.
    #[serde(default, skip)]
    pub callbacks: Vec<Child>,
}

// Implement a custom Clone, as the child processes don't implement Clone.
impl Clone for InternalState {
    fn clone(&self) -> Self {
        InternalState {
            inner: self.inner.clone(),
            shutdown: self.shutdown.clone(),
            ..Default::default()
        }
    }
}

// Implement a custom PartialEq, as the child processes don't ipmlement PartialEq.
impl Eq for InternalState {}
impl PartialEq for InternalState {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner && self.shutdown == other.shutdown
    }
}

impl InternalState {
    /// Create a new default state.
    pub fn new() -> InternalState {
        let mut state = InternalState::default();
        state.create_group(PUEUE_DEFAULT_GROUP);
        state
    }

    pub fn tasks(&self) -> &BTreeMap<usize, Task> {
        &self.inner.tasks
    }

    pub fn tasks_mut(&mut self) -> &mut BTreeMap<usize, Task> {
        &mut self.inner.tasks
    }

    pub fn groups(&self) -> &BTreeMap<String, Group> {
        &self.inner.groups
    }

    pub fn groups_mut(&mut self) -> &mut BTreeMap<String, Group> {
        &mut self.inner.groups
    }

    /// Add a new task
    pub fn add_task(&mut self, mut task: Task) -> usize {
        let next_id = match self.tasks().keys().max() {
            None => 0,
            Some(id) => id + 1,
        };
        task.id = next_id;
        self.tasks_mut().insert(next_id, task);

        next_id
    }

    /// A small helper to change the status of a specific task.
    pub fn change_status(&mut self, id: usize, new_status: TaskStatus) {
        if let Some(ref mut task) = self.tasks_mut().get_mut(&id) {
            task.status = new_status;
        };
    }

    /// Add a new group to the daemon. \
    /// This also check if the given group already exists.
    /// Create a state.group entry and a settings.group entry, if it doesn't.
    pub fn create_group(&mut self, name: &str) -> &mut Group {
        self.groups_mut().entry(name.into()).or_insert(Group {
            status: GroupStatus::Running,
            parallel_tasks: 1,
        })
    }

    /// Remove a group.
    /// This also iterates through all tasks and sets any tasks' group
    /// to the `default` group if it matches the deleted group.
    pub fn remove_group(&mut self, group: &str) -> Result<(), Error> {
        if group.eq(PUEUE_DEFAULT_GROUP) {
            return Err(Error::Generic(
                "You cannot remove the default group.".into(),
            ));
        }

        self.groups_mut().remove(group);

        Ok(())
    }

    /// Set the group status (running/paused) for all groups including the default queue.
    pub fn set_status_for_all_groups(&mut self, status: GroupStatus) {
        for (_, group) in self.groups_mut().iter_mut() {
            group.status = status;
        }
    }

    /// Get all ids of task inside a specific group.
    pub fn task_ids_in_group(&self, group: &str) -> Vec<usize> {
        self.inner
            .tasks
            .iter()
            .filter(|(_, task)| task.group.eq(group))
            .map(|(id, _)| *id)
            .collect()
    }

    /// This checks, whether some tasks match the expected filter criteria. \
    /// The first result is the list of task_ids that match these statuses. \
    /// The second result is the list of task_ids that don't match these statuses. \
    ///
    /// By default, this checks all tasks in the current state. If a list of task_ids is
    /// provided as the third parameter, only those tasks will be checked.
    pub fn filter_tasks<F>(&self, condition: F, task_ids: Option<Vec<usize>>) -> FilteredTasks
    where
        F: Fn(&Task) -> bool,
    {
        self.inner.filter_tasks(condition, task_ids)
    }

    /// Same as [State::filter_tasks], but only checks for tasks of a specific group.
    pub fn filter_tasks_of_group<F>(&self, condition: F, group: &str) -> FilteredTasks
    where
        F: Fn(&Task) -> bool,
    {
        self.inner.filter_tasks_of_group(condition, group)
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
            .tasks()
            .iter()
            .filter(|(_, task)| {
                task.dependencies.contains(task_id)
                    && !matches!(task.status, TaskStatus::Done { .. })
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
    /// whole daemon on a failed tasks. This function wraps that logic and decides if anything
    /// should be paused depending on the current settings.
    ///
    /// `group` should be the name of the failed task.
    pub fn pause_on_failure(&mut self, settings: &Settings, group: &str) {
        if settings.daemon.pause_group_on_failure {
            if let Some(group) = self.groups_mut().get_mut(group) {
                group.status = GroupStatus::Paused;
            }
        } else if settings.daemon.pause_all_on_failure {
            self.set_status_for_all_groups(GroupStatus::Paused);
        }
    }

    /// Save the current state to disk. \
    /// We do this to restore in case of a crash. \
    /// If log == true, the file will be saved with a time stamp.
    ///
    /// In comparison to the daemon -> client communication, the state is saved
    /// as JSON for readability and debugging purposes.
    pub fn save(&self, settings: &Settings) -> Result<()> {
        let serialized =
            serde_json::to_string(&self.inner).context("Failed to serialize state:")?;

        let path = settings.shared.pueue_directory();
        let mut temp = path.join("state.json.partial");
        let mut real = path.join("state.json");

        if settings.daemon.compress_status_file {
            temp = path.join("state.json.gz.partial");
            real = path.join("state.json.gz");

            let file = if temp.exists() {
                File::open(&temp)?
            } else {
                File::create(&temp)?
            };

            let mut encoder = flate2::write::GzEncoder::new(file, Compression::default());
            encoder.write_all(serialized.as_bytes())?;
        } else {
            // Write to temporary log file first, to prevent loss due to crashes.
            std::fs::write(&temp, serialized)
                .context("Failed to write temp file while saving state.")?;
        }

        // Overwrite the original with the temp file, if everything went fine.
        std::fs::rename(&temp, &real)
            .context("Failed to overwrite old state while saving state")?;

        debug!("State saved at: {real:?}");

        Ok(())
    }

    /// Restore the last state from a previous session. \
    /// The state is stored as json in the `pueue_directory`.
    ///
    /// If the state cannot be deserialized, an empty default state will be used instead. \
    /// All groups with queued tasks will be automatically paused to prevent unwanted execution.
    pub fn restore_state(settings: &Settings) -> Result<Option<InternalState>> {
        let pueue_directory = settings.shared.pueue_directory();
        let mut path = pueue_directory.join("state.json");

        if settings.daemon.compress_status_file {
            path = pueue_directory.join("state.json.gz");
        }

        // Ignore if the file doesn't exist. It doesn't have to.
        if !path.exists() {
            info!("Couldn't find state from previous session at location: {path:?}");
            return Ok(None);
        }
        info!("Restoring state");

        // Try to load the file.
        let data = if settings.daemon.compress_status_file {
            let file = File::open(path)?;
            let mut decoder = flate2::read::GzDecoder::new(file);
            let mut data = String::new();
            decoder.read_to_string(&mut data)?;

            data
        } else {
            read_to_string(&path).context("State restore: Failed to read file:\n\n{}")?
        };

        // Try to deserialize the state file.
        let state: State = serde_json::from_str(&data).context("Failed to deserialize state.")?;

        let mut state = InternalState {
            inner: state,
            ..Default::default()
        };

        // Restore all tasks.
        // While restoring the tasks, check for any invalid/broken stati.
        for (_, task) in state.inner.tasks.iter_mut() {
            // Handle ungraceful shutdowns while executing tasks.
            if let TaskStatus::Running { start, enqueued_at }
            | TaskStatus::Paused { start, enqueued_at } = task.status
            {
                info!(
                    "Setting task {} with previous status {:?} to new status {:?}",
                    task.id,
                    task.status,
                    TaskResult::Killed
                );
                task.status = TaskStatus::Done {
                    start,
                    end: Local::now(),
                    enqueued_at,
                    result: TaskResult::Killed,
                };
            }

            // Handle crash during editing of the task command.
            if matches!(task.status, TaskStatus::Locked { .. }) {
                task.status = TaskStatus::Stashed { enqueue_at: None };
            }

            // Go trough all tasks and set all groups that are no longer
            // listed in the configuration file to the default.
            let group = match state.inner.groups.get_mut(&task.group) {
                Some(group) => group,
                None => {
                    task.group = PUEUE_DEFAULT_GROUP.into();
                    state
                        .inner
                        .groups
                        .entry(PUEUE_DEFAULT_GROUP.into())
                        .or_insert(Group {
                            status: GroupStatus::Running,
                            parallel_tasks: 1,
                        })
                }
            };

            // If there are any queued tasks, pause the group.
            // This should prevent any unwanted execution of tasks due to a system crash.
            if let TaskStatus::Queued { .. } = task.status {
                info!(
                    "Pausing group {} to prevent unwanted execution of previous tasks",
                    &task.group
                );
                group.status = GroupStatus::Paused;
            }
        }

        Ok(Some(state))
    }
}
