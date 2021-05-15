use pueue_lib::log::clean_log_handles;
use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use crate::network::response_helper::*;

/// Invoked when calling `pueue remove`.
/// Remove tasks from the queue.
/// We have to ensure that those tasks aren't running!
pub fn remove(task_ids: Vec<usize>, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    let statuses = vec![
        TaskStatus::Queued,
        TaskStatus::Stashed,
        TaskStatus::Done,
        TaskStatus::Locked,
    ];
    let (mut not_running, mut running) = state.tasks_in_statuses(statuses, Some(task_ids));

    // Don't delete tasks, if there are other tasks that depend on this one.
    // However, we allow to delete those tasks, if they're supposed to be deleted as well.
    for task_id in not_running.clone() {
        if !state.is_task_removable(&task_id, &not_running) {
            running.push(task_id);
            not_running.retain(|id| id != &task_id);
        };
    }

    for task_id in &not_running {
        state.tasks.remove(task_id);

        clean_log_handles(*task_id, &state.settings.shared.pueue_directory());
    }

    let text = "Tasks removed from list";
    let response = compile_task_response(text, not_running, running);
    create_success_message(response)
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::super::fixtures::*;
    use super::*;

    #[test]
    fn normal_remove() {
        let tempdir = TempDir::new("pueue_test").expect("Failed to create test pueue directory");
        let state = get_stub_state(tempdir.into_path());

        // 3 and 4 aren't allowed to be removed, since they're running.
        // The rest will succeed.
        let message = remove(vec![0, 1, 2, 3, 4], &state);

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
        let tempdir = TempDir::new("pueue_test").expect("Failed to create test pueue directory");
        let state = get_stub_state(tempdir.into_path());

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
        let message = remove(vec![1], &state);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "The command failed for tasks: 1");
        };

        {
            let state = state.lock().unwrap();
            assert_eq!(state.tasks.len(), 7);
        }

        // Make sure we cannot remove a task with recursive dependencies.
        let message = remove(vec![1, 5], &state);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "The command failed for tasks: 1, 5");
        };

        {
            let state = state.lock().unwrap();
            assert_eq!(state.tasks.len(), 7);
        }

        // Make sure we can remove tasks with dependencies if all dependencies are specified.
        let message = remove(vec![1, 5, 6], &state);

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
