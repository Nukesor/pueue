use log::{error, info, warn};

use pueue_lib::network::message::{Signal, TaskSelection};
use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;

use crate::ok_or_shutdown;
use crate::platform::process_helper::*;
use crate::state_helper::save_state;
use crate::task_handler::{Shutdown, TaskHandler};

impl TaskHandler {
    /// Kill specific tasks or groups.
    ///
    /// By default, this kills tasks with Rust's subprocess handling "kill" logic.
    /// However, the user can decide to send unix signals to the processes as well.
    ///
    /// `kill_children` Kill all direct child processes as well
    /// `pause_groups` If `group` or `all` is given, the groups should be paused under some
    ///     circumstances. This is mostly to prevent any further task execution during an emergency
    /// `signal` Don't kill the task as usual, but rather send a unix process signal.
    pub fn kill(
        &mut self,
        tasks: TaskSelection,
        kill_children: bool,
        pause_groups: bool,
        signal: Option<Signal>,
    ) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();
        // Get the keys of all tasks that should be resumed
        let task_ids = match tasks {
            TaskSelection::TaskIds(task_ids) => task_ids,
            TaskSelection::Group(group) => {
                // Ensure that a given group exists. (Might not happen due to concurrency)
                if !state.groups.contains_key(&group) {
                    return;
                }
                // Pause this specific group.
                if pause_groups {
                    state.groups.insert(group.clone(), GroupStatus::Paused);
                }
                info!("Killing tasks of group {}", &group);

                let (matching, _) = state.filter_tasks_of_group(
                    |task| matches!(task.status, TaskStatus::Running | TaskStatus::Paused),
                    &group,
                );
                matching
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
                self.send_internal_signal(task_id, signal, kill_children);
            } else {
                self.kill_task(task_id, kill_children);
            }
        }

        ok_or_shutdown!(self, save_state(&state));
    }

    /// Send a signal to a specific child process.
    /// This is a wrapper around [send_internal_signal_to_child], which does a little bit of
    /// additional error handling.
    pub fn send_internal_signal(&mut self, task_id: usize, signal: Signal, send_to_children: bool) {
        let child = match self.children.get_child_mut(task_id) {
            Some(child) => child,
            None => {
                warn!("Tried to kill non-existing child: {}", task_id);
                return;
            }
        };

        if let Err(err) = send_internal_signal_to_child(child, signal, send_to_children) {
            warn!(
                "Failed to send signal to task {} with error: {}",
                task_id, err
            );
        };
    }

    /// Kill a specific task and handle it accordingly.
    /// Triggered on `reset` and `kill`.
    pub fn kill_task(&mut self, task_id: usize, kill_children: bool) {
        if let Some(mut child) = self.children.get_child_mut(task_id) {
            kill_child(task_id, child, kill_children);
        } else {
            warn!("Tried to kill non-existing child: {}", task_id);
        }
    }
}
