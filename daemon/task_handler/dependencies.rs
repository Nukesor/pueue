use super::*;

impl TaskHandler {
    /// Ensure that no `Queued` tasks have any failed dependencies.
    /// Otherwise set their status to `Done` and result to `DependencyFailed`.
    pub fn check_failed_dependencies(&mut self) {
        // Clone the state ref, so we don't have two mutable borrows later on.
        let state_ref = self.state.clone();
        let mut state = state_ref.lock().unwrap();

        // Get id's of all tasks with failed dependencies
        let has_failed_deps: Vec<_> = state
            .tasks
            .iter()
            .filter(|(_, task)| task.status == TaskStatus::Queued && !task.dependencies.is_empty())
            .filter_map(|(id, task)| {
                // At this point we got all queued tasks with dependencies.
                // Go through all dependencies and ensure they didn't fail.
                let failed = task
                    .dependencies
                    .iter()
                    .flat_map(|id| state.tasks.get(id))
                    .filter(|task| task.failed())
                    .map(|task| task.id)
                    .next();

                failed.map(|f| (*id, f))
            })
            .collect();

        // Update the state of all tasks with failed dependencies.
        for (id, _) in has_failed_deps {
            // Get the task's group, since we have to check if it's paused.
            let group = if let Some(task) = state.tasks.get(&id) {
                task.group.clone()
            } else {
                continue;
            };

            // Only update the status, if the group isn't paused.
            // This allows users to fix and restart dependencies in-place without
            // breaking the dependency chain.
            if matches!(state.groups.get(&group).unwrap(), GroupStatus::Paused) {
                continue;
            }

            let task = state.tasks.get_mut(&id).unwrap();
            task.status = TaskStatus::Done;
            task.result = Some(TaskResult::DependencyFailed);
            task.start = Some(Local::now());
            task.end = Some(Local::now());
            self.spawn_callback(task);
        }
    }
}
