use ::anyhow::Result;
use ::chrono::prelude::*;
use ::log::{error, info};
use ::serde_derive::{Deserialize, Serialize};
use ::std::collections::BTreeMap;
use ::std::fs;
use ::std::path::{Path, PathBuf};
use ::std::sync::{Arc, Mutex};
use ::std::time::SystemTime;

use crate::settings::Settings;
use crate::task::{Task, TaskResult, TaskStatus};

pub type SharedState = Arc<Mutex<State>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    max_id: usize,
    pub settings: Settings,
    pub running: bool,
    pub tasks: BTreeMap<usize, Task>,
}

impl State {
    pub fn new(settings: &Settings) -> State {
        let mut state = State {
            max_id: 0,
            settings: settings.clone(),
            running: true,
            tasks: BTreeMap::new(),
        };
        state.restore();
        state
    }

    pub fn add_task(&mut self, mut task: Task) -> usize {
        task.id = self.max_id;
        self.tasks.insert(self.max_id, task);
        self.max_id += 1;
        self.save();
        self.max_id - 1
    }

    /// Search and return the next task to be started.
    /// Precondition for a task to be started:
    /// - is in Queued state
    /// - has all its dependencies in `Done` state
    pub fn get_next_task_id(&mut self) -> Option<usize> {
        return self
            .tasks
            .iter()
            .filter(|(_, task)| task.status == TaskStatus::Queued)
            .filter(|(_, task)| {
                task.dependencies
                    .iter()
                    .flat_map(|id| self.tasks.get(id))
                    .all(|task| task.status == TaskStatus::Done)
            })
            .next()
            .map(|(id, _)| *id);
    }

    pub fn change_status(&mut self, id: usize, new_status: TaskStatus) {
        if let Some(ref mut task) = self.tasks.get_mut(&id) {
            task.status = new_status;
            self.save();
        };
    }

    pub fn set_enqueue_at(&mut self, id: usize, enqueue_at: Option<DateTime<Local>>) {
        if let Some(ref mut task) = self.tasks.get_mut(&id) {
            task.enqueue_at = enqueue_at;
        }
    }

    /// This checks, whether the given task_ids are in the specified statuses.
    /// The first result is the list of task_ids that match these statuses.
    /// The second result is the list of task_ids that don't match these statuses.
    ///
    /// Additionally, a list of task_ids can be specified to only run the check
    /// on a subset of all tasks.
    pub fn tasks_in_statuses(
        &mut self,
        statuses: Vec<TaskStatus>,
        task_ids: Option<Vec<usize>>,
    ) -> (Vec<usize>, Vec<usize>) {
        let task_ids = match task_ids {
            Some(ids) => ids,
            None => self.tasks.keys().cloned().collect(),
        };

        let mut matching = Vec::new();
        let mut mismatching = Vec::new();

        // Filter all task id's that match the provided statuses.
        for task_id in task_ids.iter() {
            // Check whether the task exists
            match self.tasks.get(&task_id) {
                None => {
                    mismatching.push(*task_id);
                    continue;
                }
                Some(task) => {
                    // Check whether the task status matches the specified statuses
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

    pub fn reset(&mut self) {
        self.backup();
        self.running = true;
        self.max_id = 0;
        self.tasks = BTreeMap::new();
        self.save();
    }

    /// Convenience wrapper around save_to_file
    pub fn save(&mut self) {
        self.save_to_file(false);
    }

    /// Save the current current state in a file with a timestamp
    /// At the same time remove old state logs from the log directory
    /// This function is called, when large changes to the state are applied, e.g. clean/reset
    pub fn backup(&mut self) {
        self.save_to_file(true);
        if let Err(error) = self.rotate() {
            error!("Failed to rotate files: {:?}", error);
        };
    }

    /// Save the current state to disk.
    /// We do this to restore in case of a crash
    /// If log == true, the file will be saved with a time stamp
    ///
    /// In comparison to the daemon -> client communication, the state is saved
    /// as JSON for better readability and debug purposes
    fn save_to_file(&mut self, log: bool) {
        let serialized = serde_json::to_string(&self);
        if let Err(error) = serialized {
            error!("Failed to serialize state: {:?}", error);
            return;
        }

        let serialized = serialized.unwrap();

        let path = Path::new(&self.settings.daemon.pueue_directory);
        let temp: PathBuf;
        let real: PathBuf;
        if log {
            let path = path.join("log");
            let now: DateTime<Utc> = Utc::now();
            let time = now.format("%Y-%m-%d_%H-%M-%S");
            temp = path.join(format!("{}_backup.json.partial", time));
            real = path.join(format!("{}_state.json", time));
        } else {
            temp = path.join("state.json.partial");
            real = path.join("state.json");
        }

        // Write to temporary log file first, to prevent loss due to crashes
        if let Err(error) = fs::write(&temp, serialized) {
            error!(
                "Failed to write log to directory. File permissions? Error: {:?}",
                error
            );
            return;
        }

        // Overwrite the original with the temp file, if everything went fine
        if let Err(error) = fs::rename(&temp, real) {
            error!(
                "Failed to overwrite old log file. File permissions? Error: {:?}",
                error
            );
            return;
        }
    }

    /// Restore the last state from a previous session
    /// The state is stored as json in the log directory
    fn restore(&mut self) {
        let path = Path::new(&self.settings.daemon.pueue_directory).join("state.json");

        // Ignore if the file doesn't exist. It doesn't have to.
        if !path.exists() {
            info!(
                "Couldn't find state from previous session at location: {:?}",
                path
            );
            return;
        }
        info!("Start restoring state");

        // Try to load the file
        let data = fs::read_to_string(&path);
        if let Err(error) = data {
            error!("Failed to read previous state log: {:?}", error);
            return;
        }
        let data = data.unwrap();

        // Try to deserialize the state file
        let deserialized: Result<State, serde_json::error::Error> = serde_json::from_str(&data);
        if let Err(error) = deserialized {
            error!("Failed to deserialize previous state log: {:?}", error);
            return;
        }
        let mut state = deserialized.unwrap();

        // Restore the state from the file
        // While restoring the tasks, check for any invalid/broken stati
        for (task_id, task) in state.tasks.iter_mut() {
            // Handle ungraceful shutdowns while executing tasks
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
            // Crash during editing of the task command
            if task.status == TaskStatus::Locked {
                task.status = TaskStatus::Stashed;
            }
            // If there are any queued tasks, pause the daemon
            // This should prevent any unwanted execution of tasks due to a system crash
            if task.status == TaskStatus::Queued {
                info!("Pausing daemon to prevent unwanted execution of previous tasks");
                state.running = false;
            }

            self.tasks.insert(*task_id, task.clone());
        }

        self.running = state.running;
        self.max_id = state.max_id;
    }

    /// Remove old logs that aren't needed any longer
    fn rotate(&mut self) -> Result<()> {
        let path = Path::new(&self.settings.daemon.pueue_directory);
        let path = path.join("log");

        // Get all log files in the directory with their respective system time
        let mut entries: BTreeMap<SystemTime, PathBuf> = BTreeMap::new();
        let mut directory_list = fs::read_dir(path)?;
        while let Some(Ok(entry)) = directory_list.next() {
            let path = entry.path();

            let metadata = entry.metadata()?;
            let time = metadata.modified()?;
            entries.insert(time, path);
        }

        // Remove all files aove the threshold
        // Old files are removed first (implictly by the BTree order)
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
