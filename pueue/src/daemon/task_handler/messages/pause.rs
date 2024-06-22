use log::{error, info};

use pueue_lib::network::message::TaskSelection;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;

use crate::daemon::state_helper::{save_state, LockedState};
use crate::daemon::task_handler::{ProcessAction, Shutdown, TaskHandler};
use crate::ok_or_shutdown;

impl TaskHandler {
    /// Pause specific tasks or groups.
    ///
    /// `wait` decides, whether running tasks will kept running until they finish on their own.
    pub fn pause(&mut self, selection: TaskSelection, wait: bool) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();

        // Get the keys of all tasks that should be paused
        let keys: Vec<usize> = match selection {
            TaskSelection::TaskIds(task_ids) => task_ids,
            TaskSelection::Group(group_name) => {
                // Ensure that a given group exists. (Might not happen due to concurrency)
                let group = match state.groups.get_mut(&group_name) {
                    Some(group) => group,
                    None => return,
                };

                // Pause a specific group.
                group.status = GroupStatus::Paused;
                info!("Pausing group {group_name}");

                let filtered_tasks = state.filter_tasks_of_group(
                    |task| matches!(task.status, TaskStatus::Running),
                    &group_name,
                );

                filtered_tasks.matching_ids
            }
            TaskSelection::All => {
                // Pause all groups, since we're pausing the whole daemon.
                state.set_status_for_all_groups(GroupStatus::Paused);

                info!("Pausing everything");
                state.children.all_task_ids()
            }
        };

        // Pause all tasks that were found.
        if !wait {
            for id in keys {
                self.pause_task(&mut state, id);
            }
        }

        ok_or_shutdown!(self, save_state(&state, &self.settings));
    }
    /// Pause a specific task.
    /// Send a signal to the process to actually pause the OS process.
    fn pause_task(&mut self, state: &mut LockedState, id: usize) {
        match self.perform_action(id, ProcessAction::Pause) {
            Err(err) => error!("Failed pausing task {id}: {err:?}"),
            Ok(success) => {
                if success {
                    state.change_status(id, TaskStatus::Paused);
                }
            }
        }
    }
}
