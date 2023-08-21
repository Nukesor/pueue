use anyhow::{bail, Result};

use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;
use pueue_lib::state::FilteredTasks;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::client::commands::edit::edit_task_properties;
use crate::client::commands::get_state;

/// When restarting tasks, the remote state is queried and a [AddMessage]
/// is create from the existing task in the state.
///
/// This is done on the client-side, so we can easily edit the task before restarting it.
/// It's also necessary to get all failed tasks, in case the user specified the `--all-failed` flag.
#[allow(clippy::too_many_arguments)]
pub async fn restart(
    stream: &mut GenericStream,
    task_ids: Vec<usize>,
    all_failed: bool,
    failed_in_group: Option<String>,
    start_immediately: bool,
    stashed: bool,
    in_place: bool,
    edit_command: bool,
    edit_path: bool,
    edit_label: bool,
) -> Result<()> {
    let new_status = if stashed {
        TaskStatus::Stashed { enqueue_at: None }
    } else {
        TaskStatus::Queued
    };

    let state = get_state(stream).await?;

    // Filter to get done tasks
    let done_filter = |task: &Task| task.is_done();

    let filtered_tasks = if all_failed || failed_in_group.is_some() {
        // Either all failed tasks or all failed tasks of a specific group need to be restarted.

        // First we have to get all finished tasks (Done)
        let filtered_tasks = if let Some(group) = failed_in_group {
            state.filter_tasks_of_group(done_filter, &group)
        } else {
            state.filter_tasks(done_filter, None)
        };

        // now pick the failed tasks
        let failed = filtered_tasks
            .matching_ids
            .into_iter()
            .filter(|task_id| {
                let task = state.tasks.get(task_id).unwrap();
                !matches!(task.status, TaskStatus::Done(TaskResult::Success))
            })
            .collect();

        // We return an empty vec for the mismatching tasks, since there shouldn't be any.
        // Any User provided ids are ignored in this mode.
        FilteredTasks {
            matching_ids: failed,
            ..Default::default()
        }
    } else if task_ids.is_empty() {
        bail!("Please provide the ids of the tasks you want to restart.");
    } else {
        state.filter_tasks(done_filter, Some(task_ids))
    };

    // Build a RestartMessage, if the tasks should be replaced instead of creating a copy of the
    // original task. This is only important, if replace is `True`.
    let mut restart_message = RestartMessage {
        tasks: Vec::new(),
        stashed,
        start_immediately,
    };

    // Go through all Done commands we found and restart them
    for task_id in &filtered_tasks.matching_ids {
        let task = state.tasks.get(task_id).unwrap();
        let mut new_task = Task::from_task(task);
        new_task.status = new_status.clone();

        // Edit any properties, if requested.
        let edited_props = edit_task_properties(
            &task.command,
            &task.path,
            &task.label,
            edit_command,
            edit_path,
            edit_label,
        )?;

        // Add the tasks to the singular message, if we want to restart the tasks in-place.
        // And continue with the next task. The message will then be sent after the for loop.
        if in_place {
            restart_message.tasks.push(TaskToRestart {
                task_id: *task_id,
                command: edited_props.command,
                path: edited_props.path,
                label: edited_props.label,
                delete_label: edited_props.delete_label,
            });

            continue;
        }

        // In case we don't do in-place restarts, we have to add a new task.
        // Create a AddMessage to send the task to the daemon from the updated info and the old task.
        let add_task_message = AddMessage {
            command: edited_props.command.unwrap_or_else(|| task.command.clone()),
            path: edited_props.path.unwrap_or_else(|| task.path.clone()),
            envs: task.envs.clone(),
            start_immediately,
            stashed,
            group: task.group.clone(),
            enqueue_at: None,
            dependencies: Vec::new(),
            priority: Some(task.priority),
            label: edited_props.label.or_else(|| task.label.clone()),
            print_task_id: false,
        };

        // Send the cloned task to the daemon and abort on any failure messages.
        send_message(add_task_message, stream).await?;
        if let Message::Failure(message) = receive_message(stream).await? {
            bail!(message);
        };
    }

    // Send the singular in-place restart message to the daemon.
    if in_place {
        send_message(restart_message, stream).await?;
        if let Message::Failure(message) = receive_message(stream).await? {
            bail!(message);
        };
    }

    if !filtered_tasks.matching_ids.is_empty() {
        println!("Restarted tasks: {:?}", filtered_tasks.matching_ids);
    }
    if !filtered_tasks.non_matching_ids.is_empty() {
        println!(
            "Couldn't restart tasks: {:?}",
            filtered_tasks.non_matching_ids
        );
    }

    Ok(())
}
