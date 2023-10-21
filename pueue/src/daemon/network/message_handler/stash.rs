use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use crate::daemon::network::response_helper::*;

/// Invoked when calling `pueue stash`.
/// Stash specific queued tasks.
/// They won't be executed until they're enqueued or explicitely started.
pub fn stash(task_ids: Vec<usize>, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    let filtered_tasks = state.filter_tasks(
        |task| matches!(task.status, TaskStatus::Queued | TaskStatus::Locked),
        Some(task_ids),
    );

    for task_id in &filtered_tasks.matching_ids {
        if let Some(ref mut task) = state.tasks.get_mut(task_id) {
            task.status = TaskStatus::Stashed { enqueue_at: None };
            task.enqueued_at = None;
        }
    }

    compile_task_response("Tasks are stashed", filtered_tasks)
}
