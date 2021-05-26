use pueue_lib::log::clean_log_handles;
use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::{TaskResult, TaskStatus};

use super::*;
use crate::ok_or_return_failure_message;

/// Invoked when calling `pueue clean`.
/// Remove all failed or done tasks from the state.
pub fn clean(message: CleanMessage, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    ok_or_return_failure_message!(state.backup());

    let (matching, _) = state.tasks_in_statuses(vec![TaskStatus::Done], None);

    for task_id in &matching {
        // Ensure the task is removable, i.e. there are no dependant tasks.
        if !state.is_task_removable(task_id, &[]) {
            continue;
        }
        // Check if we should ignore this task, if only successful tasks should be removed.
        if message.successful_only {
            if let Some(task) = state.tasks.get(task_id) {
                if task.result != Some(TaskResult::Success) {
                    continue;
                }
            }
        }
        let _ = state.tasks.remove(task_id).unwrap();
        clean_log_handles(*task_id, &state.settings.shared.pueue_directory());
    }

    ok_or_return_failure_message!(state.save());

    if message.successful_only {
        create_success_message("All successfully finished tasks have been removed")
    } else {
        create_success_message("All finished tasks have been removed")
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    use pretty_assertions::assert_eq;
    use tempdir::TempDir;

    fn get_message(successful_only: bool) -> CleanMessage {
        CleanMessage { successful_only }
    }

    fn get_clean_test_state() -> (SharedState, TempDir) {
        let (state, tempdir) = get_state();

        {
            let mut state = state.lock().unwrap();
            let mut task = get_stub_task("0", TaskStatus::Done);
            task.result = Some(TaskResult::Success);
            state.add_task(task);

            let mut task = get_stub_task("1", TaskStatus::Done);
            task.result = Some(TaskResult::Failed(1));
            state.add_task(task);

            let mut task = get_stub_task("2", TaskStatus::Done);
            task.result = Some(TaskResult::FailedToSpawn("error".to_string()));
            state.add_task(task);

            let mut task = get_stub_task("3", TaskStatus::Done);
            task.result = Some(TaskResult::Killed);
            state.add_task(task);

            let mut task = get_stub_task("4", TaskStatus::Done);
            task.result = Some(TaskResult::Errored);
            state.add_task(task);

            let mut task = get_stub_task("5", TaskStatus::Done);
            task.result = Some(TaskResult::DependencyFailed);
            state.add_task(task);
        }

        (state, tempdir)
    }

    #[test]
    fn clean_normal() {
        let (state, _tempdir) = get_stub_state();

        // Only task 1 will be removed, since it's the only TaskStatus with `Done`.
        let message = clean(get_message(false), &state);

        // Return message is correct
        println!("{:?}", message);
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "All finished tasks have been removed");
        };

        let state = state.lock().unwrap();
        assert_eq!(state.tasks.len(), 4);
    }

    #[test]
    fn clean_normal_for_all_results() {
        let (state, _tempdir) = get_clean_test_state();

        // All finished tasks should removed when calling default `clean`.
        let message = clean(get_message(false), &state);

        // Return message is correct
        println!("{:?}", message);
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "All finished tasks have been removed");
        };

        let state = state.lock().unwrap();
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn clean_successful_only() {
        let (state, _tempdir) = get_clean_test_state();

        // Only successfully finished tasks should get removed when
        // calling `clean` with the `successful_only` flag.
        let message = clean(get_message(true), &state);

        // Return message is correct
        println!("{:?}", message);
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "All successfully finished tasks have been removed");
        };

        // Assert that only the first entry has been deleted (TaskResult::Success)
        let state = state.lock().unwrap();
        assert_eq!(state.tasks.len(), 5);
        assert!(state.tasks.get(&0).is_none());
    }
}
