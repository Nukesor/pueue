use super::*;

use crate::ok_or_shutdown;

impl TaskHandler {
    /// Kill specific tasks or groups.
    ///
    /// 1. If task_ids is not empty, kill specific tasks.
    /// 2. If `all` is true, kill everything
    /// 3. Kill a specific group.
    ///
    /// `children` decides, whether the kill signal will be send to child processes as well.
    pub fn kill(&mut self, task_ids: Vec<usize>, group: String, all: bool, children: bool) {
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
            state.set_status_for_all_groups(GroupStatus::Paused);

            info!("Killing all running tasks");
            self.children.keys().cloned().collect()
        } else {
            // Ensure that a given group exists. (Might not happen due to concurrency)
            if !state.groups.contains_key(&group) {
                return;
            }
            // Pause a specific group.
            state.groups.insert(group.clone(), GroupStatus::Paused);
            info!("Killing tasks of group {}", &group);

            state
                .task_ids_in_group_with_stati(&group, vec![TaskStatus::Running, TaskStatus::Paused])
        };

        for task_id in task_ids {
            self.kill_task(task_id, children);
        }
        ok_or_shutdown!(self, state.save());
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
