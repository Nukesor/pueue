use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

/// Invoked when calling `pueue switch`.
/// Switch the position of two tasks in the upcoming queue.
/// We have to ensure that those tasks are either `Queued` or `Stashed`
pub fn switch(message: SwitchMessage, state: &SharedState) -> Message {
    let task_ids = vec![message.task_id_1, message.task_id_2];
    let statuses = vec![TaskStatus::Queued, TaskStatus::Stashed];
    let mut state = state.lock().unwrap();
    let (_, mismatching) = state.tasks_in_statuses(statuses, Some(task_ids.to_vec()));
    if !mismatching.is_empty() {
        return create_failure_message("Tasks have to be either queued or stashed.");
    }

    // Get the tasks. Expect them to be there, since we found no mismatch
    let mut first_task = state.tasks.remove(&task_ids[0]).unwrap();
    let mut second_task = state.tasks.remove(&task_ids[1]).unwrap();

    // Switch task ids
    let first_id = first_task.id;
    let second_id = second_task.id;
    first_task.id = second_id;
    second_task.id = first_id;

    // Put tasks back in again
    state.tasks.insert(first_task.id, first_task);
    state.tasks.insert(second_task.id, second_task);

    for (_, task) in state.tasks.iter_mut() {
        // If the task depends on both, we can just keep it as it is.
        if task.dependencies.contains(&first_id) && task.dependencies.contains(&second_id) {
            continue;
        }

        // If one of the ids is in the task's dependency list, replace it with the other one.
        if let Some(old_id) = task.dependencies.iter_mut().find(|id| *id == &first_id) {
            *old_id = second_id;
            task.dependencies.sort_unstable();
        } else if let Some(old_id) = task.dependencies.iter_mut().find(|id| *id == &second_id) {
            *old_id = first_id;
            task.dependencies.sort_unstable();
        }
    }

    create_success_message("Tasks have been switched")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempdir::TempDir;

    use super::super::fixtures::*;
    use super::*;

    fn get_message(task_id_1: usize, task_id_2: usize) -> SwitchMessage {
        SwitchMessage {
            task_id_1,
            task_id_2,
        }
    }

    fn get_test_state(path: PathBuf) -> SharedState {
        let state = get_state(path);

        {
            let mut state = state.lock().unwrap();
            let task = get_stub_task("0", TaskStatus::Queued);
            state.add_task(task);

            let task = get_stub_task("1", TaskStatus::Stashed);
            state.add_task(task);

            let task = get_stub_task("2", TaskStatus::Queued);
            state.add_task(task);

            let task = get_stub_task("3", TaskStatus::Stashed);
            state.add_task(task);

            let mut task = get_stub_task("4", TaskStatus::Queued);
            task.dependencies = vec![0, 3];
            state.add_task(task);

            let mut task = get_stub_task("5", TaskStatus::Stashed);
            task.dependencies = vec![1];
            state.add_task(task);

            let mut task = get_stub_task("6", TaskStatus::Queued);
            task.dependencies = vec![2, 3];
            state.add_task(task);
        }

        state
    }

    #[test]
    /// A normal switch between two id's works perfectly fine.
    fn switch_normal() {
        let tempdir = TempDir::new("pueue_test").expect("Failed to create test pueue directory");
        let state = get_test_state(tempdir.into_path());

        let message = switch(get_message(1, 2), &state);

        // Return message is correct
        assert!(matches!(message, Message::Success(_)));
        if let Message::Success(text) = message {
            assert_eq!(text, "Tasks have been switched");
        };

        let state = state.lock().unwrap();
        assert_eq!(state.tasks.get(&1).unwrap().command, "2");
        assert_eq!(state.tasks.get(&2).unwrap().command, "1");
    }

    #[test]
    /// If any task that is specified as dependency get's switched,
    /// all dependants need to be updated.
    fn switch_task_with_dependant() {
        let tempdir = TempDir::new("pueue_test").expect("Failed to create test pueue directory");
        let state = get_test_state(tempdir.into_path());

        switch(get_message(0, 3), &state);

        let state = state.lock().unwrap();
        assert_eq!(state.tasks.get(&4).unwrap().dependencies, vec![0, 3]);
    }

    #[test]
    /// A task with two dependencies shouldn't experience any change, if those two dependencies
    /// switched places.
    fn switch_double_dependency() {
        let tempdir = TempDir::new("pueue_test").expect("Failed to create test pueue directory");
        let state = get_test_state(tempdir.into_path());

        switch(get_message(1, 2), &state);

        let state = state.lock().unwrap();
        assert_eq!(state.tasks.get(&5).unwrap().dependencies, vec![2]);
        assert_eq!(state.tasks.get(&6).unwrap().dependencies, vec![1, 3]);
    }

    #[test]
    /// You can only switch tasks that are either stashed or queued.
    /// Everything else should result in an error message.
    fn switch_invalid() {
        let tempdir = TempDir::new("pueue_test").expect("Failed to create test pueue directory");
        let state = get_state(tempdir.into_path());

        let combinations: Vec<(usize, usize)> = vec![
            (0, 1), // Queued + Done
            (0, 3), // Queued + Stashed
            (0, 4), // Queued + Running
            (0, 5), // Queued + Paused
            (2, 1), // Stashed + Done
            (2, 3), // Stashed + Stashed
            (2, 4), // Stashed + Running
            (2, 5), // Stashed + Paused
        ];

        for ids in combinations {
            let message = switch(get_message(ids.0, ids.1), &state);

            // Assert, that we get a Failure message with the correct text.
            assert!(matches!(message, Message::Failure(_)));
            if let Message::Failure(text) = message {
                assert_eq!(text, "Tasks have to be either queued or stashed.");
            };
        }
    }
}
