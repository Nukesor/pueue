use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use crate::network::response_helper::*;

/// Invoked when calling `pueue enqueue`.
/// Enqueue specific stashed tasks.
pub fn enqueue(message: EnqueueMessage, state: &SharedState) -> Message {
    let (matching, mismatching) = {
        let mut state = state.lock().unwrap();
        let (matching, mismatching) = state.tasks_in_statuses(
            vec![TaskStatus::Stashed, TaskStatus::Locked],
            Some(message.task_ids),
        );

        for task_id in &matching {
            state.set_enqueue_at(*task_id, message.enqueue_at);
            state.change_status(*task_id, TaskStatus::Queued);
        }

        (matching, mismatching)
    };

    let text = if let Some(enqueue_at) = message.enqueue_at {
        format!(
            "Tasks will be enqueued at {}",
            enqueue_at.format("%Y-%m-%d %H:%M:%S")
        )
    } else {
        String::from("Tasks are enqueued")
    };

    let response = compile_task_response(text.as_str(), matching, mismatching);
    create_success_message(response)
}
