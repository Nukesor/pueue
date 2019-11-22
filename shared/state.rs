use ::chrono::prelude::*;
use ::serde_derive::{Deserialize, Serialize};
use ::std::collections::BTreeMap;
use ::std::sync::{Arc, Mutex};
use ::strum::IntoEnumIterator;

use crate::task::{Task, TaskStatus};

pub type SharedState = Arc<Mutex<State>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    max_id: i32,
    pub tasks: BTreeMap<i32, Task>,
}

impl State {
    pub fn new() -> State {
        return State {
            max_id: 0,
            tasks: BTreeMap::new(),
        };
    }

    pub fn add_task(&mut self, mut task: Task) -> i32 {
        task.id = self.max_id;
        self.tasks.insert(self.max_id, task);
        self.max_id += 1;
        self.max_id - 1
    }

    pub fn remove_task(&mut self, id: i32) -> Option<Task> {
        self.tasks.remove(&id)
    }

    pub fn get_task_clone(&mut self, id: i32) -> Option<Task> {
        let task = self.tasks.remove(&id);
        let clone = task.clone();
        if let Some(task) = task {
            self.tasks.insert(id, task);
        }

        return clone;
    }

    pub fn get_next_task(&mut self) -> Option<i32> {
        for (id, task) in self.tasks.iter() {
            match task.status {
                TaskStatus::Queued => {
                    return Some(*id);
                }
                _ => continue,
            }
        }
        None
    }

    pub fn change_status(&mut self, id: i32, new_status: TaskStatus) {
        if let Some(ref mut task) = self.tasks.get_mut(&id) {
            if new_status == TaskStatus::Running {
                if TaskStatus::Queued == task.status || TaskStatus::Stashed == task.status {
                    task.start = Some(Local::now());
                }
            }
            task.status = new_status;
        };
    }

    pub fn get_task_status(&mut self, id: i32) -> Option<TaskStatus> {
        if let Some(ref task) = self.tasks.get(&id) {
            return Some(task.status.clone());
        };
        None
    }

    /// This checks, whether the given task_ids are in the specified statuses.
    /// The first result is the list of task_ids that match these statuses.
    /// The second result is the list of task_ids that don't match these statuses.
    ///
    /// Additionally, if no task_ids are specified, return ids of all tasks
    pub fn tasks_in_statuses(
        &mut self,
        task_ids: Option<Vec<i32>>,
        statuses: Vec<TaskStatus>,
    ) -> (Vec<i32>, Vec<i32>) {
        let task_ids = match task_ids {
            Some(ids) => ids,
            None => self.tasks.keys().cloned().collect(),
        };

        let mut matching = Vec::new();
        let mut mismatching = Vec::new();

        // Filter all task id's that match the provided statuses.
        for task_id in task_ids.iter() {
            // We aren't interested in this task, continue
            if !self.tasks.contains_key(&task_id) {
                mismatching.push(*task_id);
                continue;
            }

            // Unwrap, since we just checked, whether it exists.
            let task = self.tasks.get(&task_id).unwrap();

            if statuses.contains(&task.status) {
                matching.push(*task_id);
            } else {
                mismatching.push(*task_id);
            }
        }

        (matching, mismatching)
    }

    /// The same as tasks_in_statuses, but with inverted statuses
    pub fn tasks_not_in_statuses(
        &mut self,
        task_ids: Option<Vec<i32>>,
        excluded_statuses: Vec<TaskStatus>,
    ) -> (Vec<i32>, Vec<i32>) {
        let mut valid_statuses = Vec::new();
        // Create a list of all valid statuses
        // (statuses that aren't the exl
        for status in TaskStatus::iter() {
            if !excluded_statuses.contains(&status) {
                valid_statuses.push(status);
            }
        }

        self.tasks_in_statuses(task_ids, valid_statuses)
    }

    pub fn reset (&mut self) {
        self.max_id = 0;
        self.tasks = BTreeMap::new();
    }
}
