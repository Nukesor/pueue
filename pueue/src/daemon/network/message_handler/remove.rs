use pueue_lib::log::clean_log_handles;
use pueue_lib::network::message::*;
use pueue_lib::settings::Settings;
use pueue_lib::state::SharedState;
use pueue_lib::task::{Task, TaskStatus};

use super::ok_or_failure_message;
use crate::daemon::network::response_helper::*;
use crate::daemon::state_helper::{is_task_removable, save_state};
use crate::ok_or_save_state_failure;

/// Invoked when calling `pueue remove`.
/// Remove tasks from the queue.
/// We have to ensure that those tasks aren't running!
pub fn remove(settings: &Settings, state: &SharedState, task_ids: Vec<usize>) -> Message {
    let mut state = state.lock().unwrap();

    // Filter all running tasks, since we cannot remove them.
    let filter = |task: &Task| {
        matches!(
            task.status,
            TaskStatus::Queued
                | TaskStatus::Stashed { .. }
                | TaskStatus::Done(_)
                | TaskStatus::Locked { .. }
        )
    };
    let mut filtered_tasks = state.filter_tasks(filter, Some(task_ids));

    // Don't delete tasks, if there are other tasks that depend on this one.
    // However, we allow to delete those tasks, if they're supposed to be deleted as well.
    for task_id in filtered_tasks.matching_ids.clone() {
        if !is_task_removable(&state, &task_id, &filtered_tasks.matching_ids) {
            filtered_tasks.non_matching_ids.push(task_id);
            filtered_tasks.matching_ids.retain(|id| id != &task_id);
        };
    }

    for task_id in &filtered_tasks.matching_ids {
        state.tasks.remove(task_id);

        clean_log_handles(*task_id, &settings.shared.pueue_directory());
    }

    ok_or_save_state_failure!(save_state(&state, settings));

    compile_task_response("Tasks removed from list", filtered_tasks)
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn normal_remove() {
        let (state, settings, _tempdir) = get_stub_state();

        // 3 and 4 aren't allowed to be removed, since they're running.
        // The rest will succeed.
        let message = remove(&settings, &state, vec![0, 1, 2, 3, 4]);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(
                text,
                "Tasks removed from list: 0, 1, 2\nThe command failed for tasks: 3, 4"
            );
        };

        let state = state.lock().unwrap();
        assert_eq!(state.tasks.len(), 2);
    }

    #[test]
    fn removal_of_dependencies() {
        let (state, settings, _tempdir) = get_stub_state();

        {
            let mut state = state.lock().unwrap();
            // Add a task with a dependency to a finished task
            let mut task = get_stub_task("5", TaskStatus::Queued);
            task.dependencies = vec![1];
            state.add_task(task);

            // Add a task depending on the previous task -> Linked dependencies
            let mut task = get_stub_task("6", TaskStatus::Queued);
            task.dependencies = vec![5];
            state.add_task(task);
        }

        // Make sure we cannot remove a task with dependencies.
        let message = remove(&settings, &state, vec![1]);

        // Return message is correct
        assert!(matches!(message, Message::Failure(_)));
        if let Message::Failure(text) = message {
            assert_eq!(text, "The command failed for tasks: 1");
        };

        {
            let state = state.lock().unwrap();
            assert_eq!(state.tasks.len(), 7);
        }

        // Make sure we cannot remove a task with recursive dependencies.
        let message = remove(&settings, &state, vec![1, 5]);

        // Return message is correct
        assert!(matches!(message, Message::Failure(_)));
        if let Message::Failure(text) = message {
            assert_eq!(text, "The command failed for tasks: 1, 5");
        };

        {
            let state = state.lock().unwrap();
            assert_eq!(state.tasks.len(), 7);
        }

        // Make sure we can remove tasks with dependencies if all dependencies are specified.
        let message = remove(&settings, &state, vec![1, 5, 6]);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "Tasks removed from list: 1, 5, 6");
        };

        {
            let state = state.lock().unwrap();
            assert_eq!(state.tasks.len(), 4);
        }
    }
}
