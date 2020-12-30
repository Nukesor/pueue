use pueue::network::message::*;
use pueue::state::SharedState;
use pueue::task::TaskStatus;

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
    let temp_id = first_task.id;
    first_task.id = second_task.id;
    second_task.id = temp_id;

    // Put tasks back in again
    state.tasks.insert(first_task.id, first_task);
    state.tasks.insert(second_task.id, second_task);

    create_success_message("Tasks have been switched")
}
