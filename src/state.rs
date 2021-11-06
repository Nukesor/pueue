use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{de, Deserializer};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Error;
use crate::settings::Settings;
use crate::task::{Task, TaskStatus};

pub const PUEUE_DEFAULT_GROUP: &str = "default";

pub type SharedState = Arc<Mutex<State>>;

/// Represents the current status of a group.
/// Each group acts as a queue and can be managed individually.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum GroupStatus {
    Running,
    Paused,
}

/// The representation of a group.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    pub status: GroupStatus,
    pub parallel_tasks: usize,
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
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct State {
    /// The current settings used by the daemon.
    pub settings: Settings,
    /// All tasks currently managed by the daemon.
    pub tasks: BTreeMap<usize, Task>,
    /// All groups with their current state a configuration.
    #[serde(deserialize_with = "deserialize_groups")]
    pub groups: BTreeMap<String, Group>,
    /// Used to store an configuration path that has been explicitely specified.
    /// Without this, the default config path will be used instead.
    pub config_path: Option<PathBuf>,
}

/// Custom group serializer, which tries to deserialize the field with the legacy representation if
/// there are any errors. That way we can recover in a smooth way from the old format.
/// This is necessary to ensure a semi-smooth transition from v1 to v2.
/// TODO: Remove in 2.1.0
fn deserialize_groups<'de, D>(deserializer: D) -> Result<BTreeMap<String, Group>, D::Error>
where
    D: Deserializer<'de>,
{
    // Do a general deserialization to Serde's `Value` type.
    // That way we don't break the deserialization state if we're unable to deserialize it into
    // our expected format.
    let value: Value = serde::Deserialize::deserialize(deserializer)?;
    let groups: Result<BTreeMap<String, Group>, serde_json::Error> =
        serde_json::from_value(value.clone());

    // If we cannot deserialize the state, this means that this is probably an old state which uses
    // the old format. Try to deserialize the old format and convert them to the new format.
    match groups {
        Ok(groups) => Ok(groups),
        Err(_) => {
            let legacy_groups: Result<BTreeMap<String, GroupStatus>, serde_json::Error> =
                serde_json::from_value(value);

            let groups = match legacy_groups {
                Ok(legacy_groups) => {
                    // Iterate over all legacy groups and create a respective new group.
                    let mut groups = BTreeMap::new();
                    for (name, _) in legacy_groups.into_iter() {
                        groups.insert(
                            name,
                            Group {
                                status: GroupStatus::Paused,
                                parallel_tasks: 1,
                            },
                        );
                    }

                    groups
                }
                Err(_) => return Err(de::Error::custom("Failed to deserialize `groups` field.")),
            };

            Ok(groups)
        }
    }
}

impl State {
    /// Create a new default state.
    pub fn new(settings: &Settings, config_path: Option<PathBuf>) -> State {
        let mut state = State {
            settings: settings.clone(),
            tasks: BTreeMap::new(),
            groups: BTreeMap::new(),
            config_path,
        };
        state.create_group(PUEUE_DEFAULT_GROUP);
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

    /// Add a new group to the daemon. \
    /// This also check if the given group already exists.
    /// Create a state.group entry and a settings.group entry, if it doesn't.
    pub fn create_group(&mut self, name: &str) -> &mut Group {
        self.groups.entry(name.into()).or_insert(Group {
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

        self.groups.remove(group);

        // Reset all tasks with removed group to the default.
        for (_, task) in self.tasks.iter_mut() {
            if task.group.eq(group) {
                task.set_default_group();
            }
        }

        Ok(())
    }

    /// Set the group status (running/paused) for all groups including the default queue.
    pub fn set_status_for_all_groups(&mut self, status: GroupStatus) {
        for (_, group) in self.groups.iter_mut() {
            group.status = status.clone();
        }
    }

    /// Get all ids of task inside a specific group.
    pub fn task_ids_in_group(&self, group: &str) -> Vec<usize> {
        self.tasks
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
    pub fn filter_tasks<F>(
        &self,
        filter: F,
        task_ids: Option<Vec<usize>>,
    ) -> (Vec<usize>, Vec<usize>)
    where
        F: Fn(&Task) -> bool,
    {
        // Either use all tasks or only the exlicitely specified ones.
        let task_ids = match task_ids {
            Some(ids) => ids,
            None => self.tasks.keys().cloned().collect(),
        };

        self.filter_task_ids(task_ids, filter)
    }

    /// Same as [tasks_in_statuses], but only checks for tasks of a specific group.
    pub fn filter_tasks_of_group<F>(&self, filter: F, group: &str) -> (Vec<usize>, Vec<usize>)
    where
        F: Fn(&Task) -> bool,
    {
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

        self.filter_task_ids(task_ids, filter)
    }

    /// Internal function used to check which of the given tasks match the provided filter.
    ///
    /// Returns a tuple of all (matching_task_ids, non_matching_task_ids).
    fn filter_task_ids<F>(&self, task_ids: Vec<usize>, filter: F) -> (Vec<usize>, Vec<usize>)
    where
        F: Fn(&Task) -> bool,
    {
        let mut matching = Vec::new();
        let mut mismatching = Vec::new();

        // Filter all task id's that match the provided statuses.
        for task_id in task_ids.iter() {
            // Check whether the task exists and save all non-existing task ids.
            match self.tasks.get(task_id) {
                None => {
                    mismatching.push(*task_id);
                    continue;
                }
                Some(task) => {
                    // Check whether the task status matches the filter.
                    if filter(task) {
                        matching.push(*task_id);
                    } else {
                        mismatching.push(*task_id);
                    }
                }
            };
        }

        (matching, mismatching)
    }
}
