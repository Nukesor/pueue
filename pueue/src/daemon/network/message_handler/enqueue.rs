use chrono::Local;
use pueue_lib::{
    format::format_datetime,
    network::message::*,
    settings::Settings,
    state::SharedState,
    success_msg,
    task::{Task, TaskStatus},
};

use crate::daemon::network::response_helper::*;

/// Invoked when calling `pueue enqueue`.
/// Enqueue specific stashed tasks.
pub fn enqueue(settings: &Settings, state: &SharedState, message: EnqueueMessage) -> Message {
    let mut state = state.lock().unwrap();
    // Get the affected task ids, based on the task selection.
    let selected_task_ids = match message.tasks {
        TaskSelection::TaskIds(ref task_ids) => state
            .tasks
            .iter()
            .filter(|(task_id, task)| {
                if !task_ids.contains(task_id) {
                    return false;
                }

                matches!(
                    task.status,
                    TaskStatus::Stashed { .. } | TaskStatus::Locked { .. }
                )
            })
            .map(|(task_id, _)| *task_id)
            .collect::<Vec<usize>>(),
        TaskSelection::Group(ref group) => state
            .tasks
            .iter()
            .filter(|(_, task)| {
                if task.group != *group {
                    return false;
                }

                matches!(
                    task.status,
                    TaskStatus::Stashed { .. } | TaskStatus::Locked { .. }
                )
            })
            .map(|(task_id, _)| *task_id)
            .collect::<Vec<usize>>(),
        TaskSelection::All => state
            .tasks
            .iter()
            .filter(|(_, task)| {
                matches!(
                    task.status,
                    TaskStatus::Stashed { .. } | TaskStatus::Locked { .. }
                )
            })
            .map(|(task_id, _)| *task_id)
            .collect::<Vec<usize>>(),
    };

    for task_id in &selected_task_ids {
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

    let matching_function = if message.enqueue_at.is_some() {
        |task: &Task| {
            matches!(
                task.status,
                TaskStatus::Stashed {
                    enqueue_at: Some(_),
                }
            )
        }
    } else {
        |task: &Task| matches!(task.status, TaskStatus::Queued { .. })
    };

    // Construct a response depending on the selected tasks.
    if let Some(enqueue_at) = &message.enqueue_at {
        let enqueue_at = format_datetime(settings, enqueue_at);

        match &message.tasks {
            TaskSelection::TaskIds(task_ids) => task_action_response_helper(
                &format!("Stashed tasks will be enqueued at {enqueue_at}"),
                task_ids.clone(),
                matching_function,
                &state,
            ),
            TaskSelection::Group(group) => {
                success_msg!("Enqueue stashed tasks of group {group} at {enqueue_at}.",)
            }
            TaskSelection::All => {
                success_msg!("Enqueue all stashed tasks at {enqueue_at}.",)
            }
        }
    } else {
        match &message.tasks {
            TaskSelection::TaskIds(task_ids) => task_action_response_helper(
                "Stashed tasks have been enqueued",
                task_ids.clone(),
                matching_function,
                &state,
            ),
            TaskSelection::Group(group) => {
                success_msg!("All stashed tasks of group \"{group}\" have been enqueued.")
            }
            TaskSelection::All => {
                success_msg!("All stashed tasks have been enqueued.")
            }
        }
    }
}
