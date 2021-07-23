use log::{error, info, warn};

use pueue_lib::network::message::TaskSelection;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::TaskStatus;

use crate::ok_or_shutdown;
use crate::state_helper::{save_state, LockedState};
use crate::task_handler::{ProcessAction, Shutdown, TaskHandler};

impl TaskHandler {
    /// Start specific tasks or groups.
    ///
    /// By default, this command only resumes tasks.
    /// However, if specific task_ids are provided, tasks can actually be force-started.
    /// Of course, they can only be started if they're in a valid status, i.e. Queued/Stashed.
    ///
    /// `children` decides, whether the resume Signal will be send to child processes as well.
    ///     Of course, this only applies to processes that are resumend and not force-spawned.
    pub fn start(&mut self, tasks: TaskSelection, children: bool) {
        let cloned_state_mutex = self.state.clone();
        let mut state = cloned_state_mutex.lock().unwrap();

        let task_ids = match tasks {
            TaskSelection::TaskIds(task_ids) => {
                // Start specific tasks.
                // This is handled differently and results in an early return, as this branch is
                // capable of force-spawning processes, instead of simply resuming tasks.
                for task_id in task_ids {
                    // Continue all children that are simply paused
                    if self.children.contains_key(&task_id) {
                        self.continue_task(&mut state, task_id, children);
                    } else {
                        // Start processes for all tasks that haven't been started yet
                        self.start_process(task_id, &mut state);
                    }
                }
                ok_or_shutdown!(self, save_state(&state));
                return;
            }
            TaskSelection::Group(group) => {
                // Ensure that a given group exists. (Might not happen due to concurrency)
                if !state.groups.contains_key(&group) {
                    return;
                }
                // Set the group to running.
                state.groups.insert(group.clone(), GroupStatus::Running);
                info!("Resuming group {}", &group);

                let (matching, _) = state.filter_tasks_of_group(
                    |task| matches!(task.status, TaskStatus::Paused),
                    &group,
                );
                matching
            }
            TaskSelection::All => {
                // Resume all groups and the default queue
                info!("Resuming everything");
                state.set_status_for_all_groups(GroupStatus::Running);

                self.children.keys().cloned().collect()
            }
        };

        // Resume all specified paused tasks
        for task_id in task_ids {
            self.continue_task(&mut state, task_id, children);
        }

        ok_or_shutdown!(self, save_state(&state));
    }

    /// Send a start signal to a paused task to continue execution.
    fn continue_task(&mut self, state: &mut LockedState, task_id: usize, children: bool) {
        // Task doesn't exist
        if !self.children.contains_key(&task_id) {
            return;
        }

        // Task is already done
        if state.tasks.get(&task_id).unwrap().is_done() {
            return;
        }

        let success = match self.perform_action(task_id, ProcessAction::Resume, children) {
            Err(err) => {
                warn!("Failed to resume task {}: {:?}", task_id, err);
                false
            }
            Ok(success) => success,
        };

        if success {
            state.change_status(task_id, TaskStatus::Running);
        }
    }
}
