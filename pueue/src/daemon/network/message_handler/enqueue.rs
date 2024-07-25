use chrono::Local;
use pueue_lib::network::message::*;
use pueue_lib::state::SharedState;
use pueue_lib::task::TaskStatus;

use crate::daemon::network::response_helper::*;

/// Invoked when calling `pueue enqueue`.
/// Enqueue specific stashed tasks.
pub fn enqueue(state: &SharedState, message: EnqueueMessage) -> Message {
    let mut state = state.lock().unwrap();
    let filtered_tasks = state.filter_tasks(
        |task| {
            matches!(
                task.status,
                TaskStatus::Stashed { .. } | TaskStatus::Locked { .. }
            )
        },
        Some(message.task_ids),
    );

    for task_id in &filtered_tasks.matching_ids {
        // We just checked that they're there and the state is locked. It's safe to unwrap.
        let task = state.tasks.get_mut(task_id).expect("Task should be there.");

        // Either specify the point of time the task should be enqueued or enqueue the task
        // immediately.
        if message.enqueue_at.is_some() {
            task.status = TaskStatus::Stashed {
                enqueue_at: message.enqueue_at,
            };
        } else {
            task.status = TaskStatus::Queued {
                enqueued_at: Local::now(),
            };
        }
    }

    let text = if let Some(enqueue_at) = message.enqueue_at {
        let enqueue_at = enqueue_at.format("%Y-%m-%d %H:%M:%S");
        format!("Tasks will be enqueued at {enqueue_at}")
    } else {
        String::from("Tasks are enqueued")
    };

    compile_task_response(&text, filtered_tasks)
}
