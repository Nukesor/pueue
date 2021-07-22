use anyhow::Context;

use super::*;

use crate::ok_or_shutdown;
use crate::state_helper::{pause_on_failure, save_state};

impl TaskHandler {
    /// Check whether there are any finished processes
    /// In case there are, handle them and update the shared state
    pub fn handle_finished_tasks(&mut self) {
        let finished = self.get_finished();

        // Nothing to do. Early return
        if finished.is_empty() {
            return;
        }

        // Clone the state ref, so we don't have two mutable borrows later on.
        let state_ref = self.state.clone();
        let mut state = state_ref.lock().unwrap();
        println!("{:?}", finished);
        println!("{:?}", &self.children.0.keys());
        println!("{:?}", &self.children.0.get("default").unwrap().keys());

        for ((task_id, group, worker_id), error) in finished.iter() {
            // Handle std::io errors on child processes.
            // I have never seen something like this, but it might happen.
            if let Some(error) = error {
                let (_taks_id, _child) = self
                    .children
                    .0
                    .get_mut(group)
                    .expect("Worker group must exist when handling finished tasks.")
                    .remove(worker_id)
                    .expect("Errored child went missing while handling finished task.");

                let group = {
                    let mut task = state.tasks.get_mut(task_id).unwrap();
                    task.status = TaskStatus::Done(TaskResult::Errored);
                    task.end = Some(Local::now());
                    self.spawn_callback(task);

                    task.group.clone()
                };
                error!("Child {} failed with io::Error: {:?}", task_id, error);

                pause_on_failure(&mut state, group);
                continue;
            }

            // Handle any tasks that exited with some kind of exit code
            let (_task_id, mut child) = self
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
                    "Failed on wait() for finished task {} with error: {:?}",
                    task_id, error
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
                let mut task = state
                    .tasks
                    .get_mut(task_id)
                    .expect("Task was removed before child process has finished!");

                task.status = TaskStatus::Done(result.clone());
                task.end = Some(Local::now());
                self.spawn_callback(task);

                task.group.clone()
            };

            if let TaskResult::Failed(_) = result {
                pause_on_failure(&mut state, group);
            }

            // Already remove the output files, if the daemon is being reset anyway
            if self.full_reset {
                clean_log_handles(*task_id, &self.pueue_directory);
            }
        }

        ok_or_shutdown!(self, save_state(&state));
    }

    /// Gather all finished tasks and sort them by finished and errored.
    /// Returns a list of finished task ids and whether they errored or not.
    fn get_finished(&mut self) -> Vec<((usize, String, usize), Option<std::io::Error>)> {
        let mut finished = Vec::new();
        for (group, children) in self.children.0.iter_mut() {
            for (worker_id, (task_id, child)) in children.iter_mut() {
                match child.try_wait() {
                    // Handle a child error.
                    Err(error) => {
                        finished.push(((*task_id, group.clone(), *worker_id), Some(error)));
                    }
                    // Child process did not exit yet
                    Ok(None) => continue,
                    Ok(_exit_status) => {
                        info!("Task {} just finished", task_id);
                        finished.push(((*task_id, group.clone(), *worker_id), None));
                    }
                }
            }
        }

        finished
    }
}
