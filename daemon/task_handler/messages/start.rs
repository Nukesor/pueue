use log::{error, info, warn};

use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;

use crate::ok_or_shutdown;
use crate::task_handler::{LockedState, ProcessAction, TaskHandler};

impl TaskHandler {
    /// Start specific tasks or groups.
    ///
    /// 1. If task_ids is not empty, start specific tasks.
    /// 2. If `all` is true, start everything.
    /// 3. Start specific group.
    ///
    /// `children` decides, whether the start signal will be send to child processes as well.
    pub fn start(&mut self, task_ids: Vec<usize>, group: String, all: bool, children: bool) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();

        // Only start specific tasks
        // This is handled separately, since this can also force-spawn processes
        if !task_ids.is_empty() {
            for id in &task_ids {
                // Continue all children that are simply paused
                if self.children.contains_key(id) {
                    self.continue_task(&mut state, *id, children);
                } else {
                    // Start processes for all tasks that haven't been started yet
                    self.start_process(*id, &mut state);
                }
            }
            ok_or_shutdown!(self, state.save());
            return;
        }

        // Get the keys of all tasks that should be resumed
        // These can either be
        // - All running tasks
        // - The paused tasks of a specific group
        // - The paused tasks of the default queue
        let keys: Vec<usize> = if all {
            // Resume all groups and the default queue
            info!("Resuming everything");
            state.set_status_for_all_groups(GroupStatus::Running);

            self.children.keys().cloned().collect()
        } else {
            // Ensure that a given group exists. (Might not happen due to concurrency)
            if !state.groups.contains_key(&group) {
                return;
            }
            // Set the group to running.
            state.groups.insert(group.clone(), GroupStatus::Running);
            info!("Resuming group {}", &group);

            state.task_ids_in_group_with_stati(&group, vec![TaskStatus::Paused])
        };

        // Resume all specified paused tasks
        for id in keys {
            self.continue_task(&mut state, id, children);
        }

        ok_or_shutdown!(self, state.save());
    }

    /// Send a start signal to a paused task to continue execution.
    fn continue_task(&mut self, state: &mut LockedState, id: usize, children: bool) {
        // Task doesn't exist
        if !self.children.contains_key(&id) {
            return;
        }

        // Task is already done
        if state.tasks.get(&id).unwrap().is_done() {
            return;
        }

        let success = match self.perform_action(id, ProcessAction::Resume, children) {
            Err(err) => {
                warn!("Failed to resume task {}: {:?}", id, err);
                false
            }
            Ok(success) => success,
        };

        if success {
            state.change_status(id, TaskStatus::Running);
        }
    }
}
