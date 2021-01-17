use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use crate::network::response_helper::*;

/// Invoked when calling `pueue stash`.
/// Stash specific queued tasks.
/// They won't be executed until they're enqueued or explicitely started.
pub fn stash(task_ids: Vec<usize>, state: &SharedState) -> Message {
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        let (matching, mismatching) =
            state.tasks_in_statuses(vec![TaskStatus::Queued, TaskStatus::Locked], Some(task_ids));

        for task_id in &matching {
            state.change_status(*task_id, TaskStatus::Stashed);
        }

        (matching, mismatching)
    };

    let text = "Tasks are stashed";
    let response = compile_task_response(text, matching, mismatching);
    create_success_message(response)
}
