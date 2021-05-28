use log::{error, info};

use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;

use crate::ok_or_shutdown;
use crate::task_handler::{LockedState, ProcessAction, TaskHandler};

impl TaskHandler {
    /// Pause specific tasks or groups.
    ///
    /// 1. If task_ids is not empty, pause specific tasks.
    /// 2. If `all` is true, pause everything.
    /// 3. Pause a specific group.
    ///
    /// `children` decides, whether the pause signal will be send to child processes as well.
    /// `wait` decides, whether running tasks will kept running until they finish on their own.
    pub fn pause(
        &mut self,
        task_ids: Vec<usize>,
        group: String,
        all: bool,
        children: bool,
        wait: bool,
    ) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();

        // Get the keys of all tasks that should be paused
        // These can either be
        // - User specified tasks
        // - All tasks
        // - Tasks of a specific group
        // Only pause specific tasks
        let keys: Vec<usize> = if !task_ids.is_empty() {
            task_ids
        } else if all {
            // Pause all groups, since we're pausing the whole daemon.
            state.set_status_for_all_groups(GroupStatus::Paused);

            info!("Pausing everything");
            self.children.keys().cloned().collect()
        } else {
            // Ensure that a given group exists. (Might not happen due to concurrency)
            if !state.groups.contains_key(&group) {
                return;
            }
            // Pause a specific group.
            state.groups.insert(group.clone(), GroupStatus::Paused);
            info!("Pausing group {}", &group);

            state.task_ids_in_group_with_stati(&group, vec![TaskStatus::Running])
        };

        // Pause all tasks that were found.
        if !wait {
            for id in keys {
                self.pause_task(&mut state, id, children);
            }
        }

        ok_or_shutdown!(self, state.save());
    }
    /// Pause a specific task.
    /// Send a signal to the process to actually pause the OS process.
    fn pause_task(&mut self, state: &mut LockedState, id: usize, children: bool) {
        match self.perform_action(id, ProcessAction::Pause, children) {
            Err(err) => error!("Failed pausing task {}: {:?}", id, err),
            Ok(success) => {
                if success {
                    state.change_status(id, TaskStatus::Paused);
                }
            }
        }
    }
}
