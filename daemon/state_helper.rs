use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::MutexGuard;
use std::time::SystemTime;

use anyhow::{Context, Result};
use chrono::prelude::*;
use log::{debug, info};

use pueue_lib::state::{Group, GroupStatus, State, PUEUE_DEFAULT_GROUP};
use pueue_lib::task::{TaskResult, TaskStatus};

pub type LockedState<'a> = MutexGuard<'a, State>;

/// Check if a task can be deleted. \
/// We have to check all dependant tasks, that haven't finished yet.
/// This is necessary to prevent deletion of tasks which are specified as a dependency.
///
/// `to_delete` A list of task ids, which should also be deleted.
///             This allows to remove dependency tasks as well as their dependants.
pub fn is_task_removable(state: &LockedState, task_id: &usize, to_delete: &[usize]) -> bool {
    // Get all task ids of any dependant tasks.
    let dependants: Vec<usize> = state
        .tasks
        .iter()
        .filter(|(_, task)| {
            task.dependencies.contains(task_id) && !matches!(task.status, TaskStatus::Done(_))
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
        .all(|task_id| is_task_removable(state, task_id, to_delete))
}

/// A small helper for handling task failures. \
/// Users can specify whether they want to pause the task's group or the
/// whole daemon on a failed tasks. This function wraps that logic and decides if anything should be
/// paused depending on the current settings.
///
/// `group` should be the name of the failed task.
pub fn pause_on_failure(state: &mut LockedState, group: &str) {
    if state.settings.daemon.pause_group_on_failure {
        if let Some(group) = state.groups.get_mut(group) {
            group.status = GroupStatus::Paused;
        }
    } else if state.settings.daemon.pause_all_on_failure {
        state.set_status_for_all_groups(GroupStatus::Paused);
    }
}

/// Do a full reset of the state.
/// This doesn't reset any processes!
pub fn reset_state(state: &mut LockedState) -> Result<()> {
    backup_state(state)?;
    state.tasks = BTreeMap::new();
    state.set_status_for_all_groups(GroupStatus::Running);

    save_state(state)
}

/// Convenience wrapper around save_to_file.
pub fn save_state(state: &State) -> Result<()> {
    save_state_to_file(state, false)
}

/// Save the current current state in a file with a timestamp.
/// At the same time remove old state logs from the log directory.
/// This function is called, when large changes to the state are applied, e.g. clean/reset.
pub fn backup_state(state: &LockedState) -> Result<()> {
    save_state_to_file(state, true)?;
    rotate_state(state).context("Failed to rotate old log files")?;
    Ok(())
}

/// Save the current state to disk. \
/// We do this to restore in case of a crash. \
/// If log == true, the file will be saved with a time stamp.
///
/// In comparison to the daemon -> client communication, the state is saved
/// as JSON for readability and debugging purposes.
fn save_state_to_file(state: &State, log: bool) -> Result<()> {
    let serialized = serde_json::to_string(&state).context("Failed to serialize state:");

    let serialized = serialized.unwrap();
    let path = state.settings.shared.pueue_directory();
    let (temp, real) = if log {
        let path = path.join("log");
        let now: DateTime<Utc> = Utc::now();
        let time = now.format("%Y-%m-%d_%H-%M-%S");
        (
            path.join(format!("{time}_state.json.partial")),
            path.join(format!("{time}_state.json")),
        )
    } else {
        (path.join("state.json.partial"), path.join("state.json"))
    };

    // Write to temporary log file first, to prevent loss due to crashes.
    fs::write(&temp, serialized).context("Failed to write temp file while saving state.")?;

    // Overwrite the original with the temp file, if everything went fine.
    fs::rename(&temp, &real).context("Failed to overwrite old state while saving state")?;

    if log {
        debug!("State backup created at: {real:?}");
    } else {
        debug!("State saved at: {real:?}");
    }

    Ok(())
}

/// Restore the last state from a previous session. \
/// The state is stored as json in the `pueue_directory`.
///
/// If the state cannot be deserialized, an empty default state will be used instead. \
/// All groups with queued tasks will be automatically paused to prevent unwanted execution.
pub fn restore_state(pueue_directory: &Path) -> Result<Option<State>> {
    let path = pueue_directory.join("state.json");

    // Ignore if the file doesn't exist. It doesn't have to.
    if !path.exists() {
        info!("Couldn't find state from previous session at location: {path:?}");
        return Ok(None);
    }
    info!("Restoring state");

    // Try to load the file.
    let data = fs::read_to_string(&path).context("State restore: Failed to read file:\n\n{}")?;

    // Try to deserialize the state file.
    let mut state: State = serde_json::from_str(&data).context("Failed to deserialize state.")?;

    // Restore all tasks.
    // While restoring the tasks, check for any invalid/broken stati.
    for (_, task) in state.tasks.iter_mut() {
        // Handle ungraceful shutdowns while executing tasks.
        if task.status == TaskStatus::Running || task.status == TaskStatus::Paused {
            info!(
                "Setting task {} with previous status {:?} to new status {:?}",
                task.id,
                task.status,
                TaskResult::Killed
            );
            task.status = TaskStatus::Done(TaskResult::Killed);
        }

        // Handle crash during editing of the task command.
        if task.status == TaskStatus::Locked {
            task.status = TaskStatus::Stashed { enqueue_at: None };
        }

        // Go trough all tasks and set all groups that are no longer
        // listed in the configuration file to the default.
        let group = match state.groups.get_mut(&task.group) {
            Some(group) => group,
            None => {
                task.set_default_group();
                state
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
        if task.status == TaskStatus::Queued {
            info!(
                "Pausing group {} to prevent unwanted execution of previous tasks",
                &task.group
            );
            group.status = GroupStatus::Paused;
        }
    }

    Ok(Some(state))
}

/// Remove old logs that aren't needed any longer.
fn rotate_state(state: &LockedState) -> Result<()> {
    let path = state.settings.shared.pueue_directory().join("log");

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
