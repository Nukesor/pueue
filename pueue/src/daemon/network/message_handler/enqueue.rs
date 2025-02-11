use chrono::Local;
use pueue_lib::{
    format::format_datetime,
    network::message::*,
    settings::Settings,
    success_msg,
    task::{Task, TaskStatus},
};

use crate::daemon::{internal_state::SharedState, network::response_helper::*};

/// Invoked when calling `pueue enqueue`.
/// Enqueue specific stashed tasks.
pub fn enqueue(settings: &Settings, state: &SharedState, message: EnqueueMessage) -> Response {
    let mut state = state.lock().unwrap();
    // Get the affected task ids, based on the task selection.
    let selected_tasks = match message.tasks {
        TaskSelection::TaskIds(ref task_ids) => state
            .tasks_mut()
            .iter_mut()
            .filter(|(task_id, task)| {
                if !task_ids.contains(task_id) {
                    return false;
                }

                matches!(
                    task.status,
                    TaskStatus::Stashed { .. } | TaskStatus::Locked { .. }
                )
            })
            .collect::<Vec<(&usize, &mut Task)>>(),
        TaskSelection::Group(ref group) => state
            .tasks_mut()
            .iter_mut()
            .filter(|(_, task)| {
                if task.group != *group {
                    return false;
                }

                matches!(
                    task.status,
                    TaskStatus::Stashed { .. } | TaskStatus::Locked { .. }
                )
            })
            .collect::<Vec<(&usize, &mut Task)>>(),
        TaskSelection::All => state
            .tasks_mut()
            .iter_mut()
            .filter(|(_, task)| {
                matches!(
                    task.status,
                    TaskStatus::Stashed { .. } | TaskStatus::Locked { .. }
                )
            })
            .collect::<Vec<(&usize, &mut Task)>>(),
    };

    for (_, task) in selected_tasks {
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
