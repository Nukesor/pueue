use log::{error, info, warn};

use pueue_lib::network::message::Signal;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;

use crate::ok_or_shutdown;
use crate::platform::process_helper::*;
use crate::state_helper::save_state;
use crate::task_handler::{Shutdown, TaskHandler};

impl TaskHandler {
    /// Kill specific tasks or groups.
    ///
    /// `task_ids` These specific ids will be killed.
    /// `all` If true, kill everything.
    /// `group` Kill a specific group.
    /// `children` Kill all direct child processes as well
    /// `pause_groups` If `group` or `all` is given, the groups should be paused under some
    ///     circumstances. This is mostly to prevent any further task execution during an emergency
    /// `signal` Don't kill the task as usual, but rather send a unix process signal.
    ///
    pub fn kill(
        &mut self,
        task_ids: Vec<usize>,
        group: String,
        all: bool,
        children: bool,
        pause_groups: bool,
        signal: Option<Signal>,
    ) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();
        // Get the keys of all tasks that should be resumed
        // These can either be
        // - Specific tasks
        // - All running tasks
        // - The paused tasks of a group
        // - The paused tasks of the default queue
        // Only pause specific tasks
        let task_ids: Vec<usize> = if !task_ids.is_empty() {
            task_ids
        } else if all {
            // Pause all running tasks
            if pause_groups {
                state.set_status_for_all_groups(GroupStatus::Paused);
            }

            info!("Killing all running tasks");
            self.children.keys().cloned().collect()
        } else {
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
        };

        for task_id in task_ids {
            if let Some(signal) = signal.clone() {
                self.send_internal_signal(task_id, signal, children);
            } else {
                self.kill_task(task_id, children);
            }
        }

        ok_or_shutdown!(self, save_state(&state));
    }

    /// Send a signal to a specific child process.
    /// This is a wrapper around [send_internal_signal_to_child], which does a little bit of
    /// additional error handling.
    pub fn send_internal_signal(&mut self, task_id: usize, signal: Signal, children: bool) {
        let child = match self.children.get_mut(&task_id) {
            Some(child) => child,
            None => {
                warn!("Tried to kill non-existing child: {}", task_id);
                return;
            }
        };

        if let Err(err) = send_internal_signal_to_child(child, signal, children) {
            warn!(
                "Failed to send signal to task {} with error: {}",
                task_id, err
            );
        };
    }

    /// Kill a specific task and handle it accordingly.
    /// Triggered on `reset` and `kill`.
    pub fn kill_task(&mut self, task_id: usize, kill_children: bool) {
        if let Some(mut child) = self.children.get_mut(&task_id) {
            kill_child(task_id, &mut child, kill_children);
        } else {
            warn!("Tried to kill non-existing child: {}", task_id);
        }
    }
}
