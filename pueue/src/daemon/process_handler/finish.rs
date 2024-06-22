use anyhow::Context;
use chrono::Local;
use log::info;
use pueue_lib::log::clean_log_handles;
use pueue_lib::task::{TaskResult, TaskStatus};

use super::*;

use crate::daemon::state_helper::{pause_on_failure, save_state, LockedState};
use crate::ok_or_shutdown;

/// Check whether there are any finished processes
/// In case there are, handle them and update the shared state
pub fn handle_finished_tasks(settings: &Settings, state: &mut LockedState) {
    // Clone the state ref, so we don't have two mutable borrows later on.
    let finished = get_finished(state);

    // Nothing to do. Early return
    if finished.is_empty() {
        return;
    }

    for ((task_id, group, worker_id), error) in finished.iter() {
        // Handle std::io errors on child processes.
        // I have never seen something like this, but it might happen.
        if let Some(error) = error {
            let (_taks_id, _child) = state
                .children
                .0
                .get_mut(group)
                .expect("Worker group must exist when handling finished tasks.")
                .remove(worker_id)
                .expect("Errored child went missing while handling finished task.");

            let group = {
                let task = state.tasks.get_mut(task_id).unwrap();
                task.status = TaskStatus::Done(TaskResult::Errored);
                task.end = Some(Local::now());
                // TODO:
                //spawn_callback(task);

                task.group.clone()
            };
            error!("Child {} failed with io::Error: {:?}", task_id, error);

            pause_on_failure(state, settings, &group);
            continue;
        }

        // Handle any tasks that exited with some kind of exit code
        let (_task_id, mut child) = state
            .children
            .0
            .get_mut(group)
            .expect("Worker group must exist when handling finished tasks.")
            .remove(worker_id)
            .expect("Child of task {} went away while handling finished task.");

        // Get the exit code of the child.
        // Errors really shouldn't happen in here, since we already checked if it's finished
        // with try_wait() before.
        let exit_code_result = child.wait();
        let exit_code = exit_code_result
            .context(format!(
                "Failed on wait() for finished task {task_id} with error: {error:?}"
            ))
            .unwrap()
            .code();

        // Processes with exit code 0 exited successfully
        // Processes with `None` have been killed by a Signal
        let result = match exit_code {
            Some(0) => TaskResult::Success,
            Some(exit_code) => TaskResult::Failed(exit_code),
            None => TaskResult::Killed,
        };

        // Update all properties on the task and get the group for later
        let group = {
            let task = state
                .tasks
                .get_mut(task_id)
                .expect("Task was removed before child process has finished!");

            task.status = TaskStatus::Done(result.clone());
            task.end = Some(Local::now());
            // TODO:
            //spawn_callback(task);

            task.group.clone()
        };

        if let TaskResult::Failed(_) = result {
            pause_on_failure(state, settings, &group);
        }

        // Already remove the output files, if the daemon is being reset anyway
        if state.full_reset {
            clean_log_handles(*task_id, &settings.shared.pueue_directory());
        }
    }

    ok_or_shutdown!(settings, state, save_state(state, settings));
}

/// Gather all finished tasks and sort them by finished and errored.
/// Returns a list of finished task ids and whether they errored or not.
fn get_finished(state: &mut LockedState) -> Vec<((usize, String, usize), Option<std::io::Error>)> {
    let mut finished = Vec::new();
    for (group, children) in state.children.0.iter_mut() {
        for (worker_id, (task_id, child)) in children.iter_mut() {
            match child.try_wait() {
                // Handle a child error.
                Err(error) => {
                    finished.push(((*task_id, group.clone(), *worker_id), Some(error)));
                }
                // Child process did not exit yet
                Ok(None) => continue,
                Ok(_exit_status) => {
                    info!("Task {task_id} just finished");
                    finished.push(((*task_id, group.clone(), *worker_id), None));
                }
            }
        }
    }

    finished
}
