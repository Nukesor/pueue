use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use crate::daemon::network::response_helper::*;

/// Invoked when calling `pueue stash`.
/// Stash specific queued tasks.
/// They won't be executed until they're enqueued or explicitely started.
pub fn stash(task_ids: Vec<usize>, state: &SharedState) -> Message {
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        let (matching, mismatching) = state.filter_tasks(
            |task| matches!(task.status, TaskStatus::Queued | TaskStatus::Locked),
            Some(task_ids),
        );

        for task_id in &matching {
            if let Some(ref mut task) = state.tasks.get_mut(task_id) {
                task.status = TaskStatus::Stashed { enqueue_at: None };
                task.enqueued_at = None;
            }
        }

        (matching, mismatching)
    };

    compile_task_response("Tasks are stashed", matching, mismatching)
}
