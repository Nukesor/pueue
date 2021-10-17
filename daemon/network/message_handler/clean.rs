use pueue_lib::log::clean_log_handles;
use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::{TaskResult, TaskStatus};

use super::*;
use crate::ok_or_return_failure_message;
use crate::state_helper::{is_task_removable, save_state};

fn construct_success_clean_message(message: CleanMessage) -> String {
    let successfull_only_fix = if message.successful_only {
        " successfully"
    } else {
        ""
    };

    let group_fix = if let Some(group) = message.group {
        format!(" in group '{}'", group)
    } else {
        String::new()
    };

    format!(
        "All{} finished tasks have been removed{}",
        successfull_only_fix, group_fix
    )
}

/// Invoked when calling `pueue clean`.
/// Remove all failed or done tasks from the state.
pub fn clean(message: CleanMessage, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    ok_or_return_failure_message!(save_state(&state));

    let (matching, _) = state.filter_tasks(|task| matches!(task.status, TaskStatus::Done(_)), None);

    for task_id in &matching {
        // Ensure the task is removable, i.e. there are no dependant tasks.
        if !is_task_removable(&state, task_id, &[]) {
            continue;
        }

        if message.successful_only || message.group.is_some() {
            if let Some(task) = state.tasks.get(task_id) {
                // Check if we should ignore this task, if only successful tasks should be removed.
                if message.successful_only
                    && !matches!(task.status, TaskStatus::Done(TaskResult::Success))
                {
                    continue;
                }

                // User's can specify a specific group to be cleaned.
                // Skip the task if that's the case and the task's group doesn't match.
                if message.group.is_some() && message.group.as_deref() != Some(&task.group) {
                    continue;
                }
            }
        }
        let _ = state.tasks.remove(task_id).unwrap();
        clean_log_handles(*task_id, &state.settings.shared.pueue_directory());
    }

    ok_or_return_failure_message!(save_state(&state));

    create_success_message(construct_success_clean_message(message))
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    fn get_message(successful_only: bool, group: Option<String>) -> CleanMessage {
        CleanMessage {
            successful_only,
            group,
        }
    }

    trait TaskAddable {
        fn add_stub_task(&mut self, id: &str, group: &str, task_result: TaskResult);
    }

    impl TaskAddable for State {
        fn add_stub_task(&mut self, id: &str, group: &str, task_result: TaskResult) {
            let task = get_stub_task_in_group(id, group, TaskStatus::Done(task_result));
            self.add_task(task);
        }
    }

    /// gets the clean test state with the required groups
    fn get_clean_test_state(groups: &[&str]) -> (SharedState, TempDir) {
        let (state, tempdir) = get_state();

        {
            let mut state = state.lock().unwrap();

            for &group in groups {
                if !state.groups.contains_key(group) {
                    state.create_group(group);
                }

                state.add_stub_task("0", group, TaskResult::Success);
                state.add_stub_task("1", group, TaskResult::Failed(1));
                state.add_stub_task("2", group, TaskResult::FailedToSpawn("error".to_string()));
                state.add_stub_task("3", group, TaskResult::Killed);
                state.add_stub_task("4", group, TaskResult::Errored);
                state.add_stub_task("5", group, TaskResult::DependencyFailed);
            }
        }

        (state, tempdir)
    }

    #[test]
    fn clean_normal() {
        let (state, _tempdir) = get_stub_state();

        // Only task 1 will be removed, since it's the only TaskStatus with `Done`.
        let message = clean(get_message(false, None), &state);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "All finished tasks have been removed");
        };

        let state = state.lock().unwrap();
        assert_eq!(state.tasks.len(), 4);
    }

    #[test]
    fn clean_normal_for_all_results() {
        let (state, _tempdir) = get_clean_test_state(&[PUEUE_DEFAULT_GROUP]);

        // All finished tasks should removed when calling default `clean`.
        let message = clean(get_message(false, None), &state);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "All finished tasks have been removed");
        };

        let state = state.lock().unwrap();
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn clean_successful_only() {
        let (state, _tempdir) = get_clean_test_state(&[PUEUE_DEFAULT_GROUP]);

        // Only successfully finished tasks should get removed when
        // calling `clean` with the `successful_only` flag.
        let message = clean(get_message(true, None), &state);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "All successfully finished tasks have been removed");
        };

        // Assert that only the first entry has been deleted (TaskResult::Success)
        let state = state.lock().unwrap();
        assert_eq!(state.tasks.len(), 5);
        assert!(state.tasks.get(&0).is_none());
    }

    #[test]
    fn clean_only_in_selected_group() {
        let (state, _tempdir) = get_clean_test_state(&[PUEUE_DEFAULT_GROUP, "other"]);

        // All finished tasks should removed in selected group (other)
        let message = clean(get_message(false, Some("other".into())), &state);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));

        if let Message::Success(text) = message {
            assert_eq!(
                text,
                "All finished tasks have been removed in group 'other'"
            );
        };

        // Assert that only the 'other' group has been cleared
        let state = state.lock().unwrap();
        assert_eq!(state.tasks.len(), 6);
        assert!(state.tasks.iter().all(|(_, task)| &task.group != "other"));
    }

    #[test]
    fn clean_only_successful_only_in_selected_group() {
        let (state, _tempdir) = get_clean_test_state(&[PUEUE_DEFAULT_GROUP, "other"]);

        // Only successfully finished tasks should removed in the 'other' group
        let message = clean(get_message(true, Some("other".into())), &state);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));

        if let Message::Success(text) = message {
            assert_eq!(
                text,
                "All successfully finished tasks have been removed in group 'other'"
            );
        };

        // Assert that only the first entry has been deleted from the 'other' group (TaskResult::Success)
        let state = state.lock().unwrap();
        assert_eq!(state.tasks.len(), 11);
        assert!(state.tasks.get(&6).is_none());
    }
}
