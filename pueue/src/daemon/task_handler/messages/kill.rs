use log::{error, info, warn};

use pueue_lib::network::message::{Signal, TaskSelection};
use pueue_lib::process_helper::*;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;

use crate::daemon::state_helper::save_state;
use crate::daemon::task_handler::{Shutdown, TaskHandler};
use crate::ok_or_shutdown;

impl TaskHandler {
    /// Kill specific tasks or groups.
    ///
    /// By default, this kills tasks with Rust's subprocess handling "kill" logic.
    /// However, the user can decide to send unix signals to the processes as well.
    ///
    /// `pause_groups` If `group` or `all` is given, the groups should be paused under some
    ///     circumstances. This is mostly to prevent any further task execution during an emergency
    /// `signal` Don't kill the task as usual, but rather send a unix process signal.
    pub fn kill(&mut self, tasks: TaskSelection, pause_groups: bool, signal: Option<Signal>) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();
        // Get the keys of all tasks that should be resumed
        let task_ids = match tasks {
            TaskSelection::TaskIds(task_ids) => task_ids,
            TaskSelection::Group(group_name) => {
                // Ensure that a given group exists. (Might not happen due to concurrency)
                let group = match state.groups.get_mut(&group_name) {
                    Some(group) => group,
                    None => return,
                };

                // Pause this specific group.
                if pause_groups {
                    group.status = GroupStatus::Paused;
                }

                // Determine all running or paused tasks in that group.
                let filtered_tasks = state.filter_tasks_of_group(
                    |task| matches!(task.status, TaskStatus::Running | TaskStatus::Paused),
                    &group_name,
                );

                info!("Killing tasks of group {group_name}");
                filtered_tasks.matching_ids
            }
            TaskSelection::All => {
                // Pause all running tasks
                if pause_groups {
                    state.set_status_for_all_groups(GroupStatus::Paused);
                }

                info!("Killing all running tasks");
                self.children.all_task_ids()
            }
        };

        for task_id in task_ids {
            if let Some(signal) = signal.clone() {
                self.send_internal_signal(task_id, signal);
            } else {
                self.kill_task(task_id);
            }
        }

        ok_or_shutdown!(self, save_state(&state, &self.settings));
    }

    /// Send a signal to a specific child process.
    /// This is a wrapper around [send_signal_to_child], which does a little bit of
    /// additional error handling.
    pub fn send_internal_signal(&mut self, task_id: usize, signal: Signal) {
        let child = match self.children.get_child_mut(task_id) {
            Some(child) => child,
            None => {
                warn!("Tried to kill non-existing child: {task_id}");
                return;
            }
        };

        if let Err(err) = send_signal_to_child(child, signal) {
            warn!("Failed to send signal to task {task_id} with error: {err}");
        };
    }

    /// Kill a specific task and handle it accordingly.
    /// Triggered on `reset` and `kill`.
    pub fn kill_task(&mut self, task_id: usize) {
        if let Some(child) = self.children.get_child_mut(task_id) {
            kill_child(task_id, child).unwrap_or_else(|err| {
                warn!("Failed to send kill to task {task_id} child process {child:?} with error {err:?}");
            })
        } else {
            warn!("Tried to kill non-existing child: {task_id}");
        }
    }
}
