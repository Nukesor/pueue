use std::collections::BTreeMap;

use chrono::Local;
use color_eyre::eyre::ContextCompat;
use pueue_lib::{
    Client, Settings,
    message::*,
    state::FilteredTasks,
    task::{Task, TaskResult, TaskStatus},
};

use crate::{
    client::commands::{edit::edit_tasks, get_state},
    internal_prelude::*,
};

/// When restarting tasks, the remote state is queried and a [AddRequest]
/// is create from the existing task in the state.
///
/// This is done on the client-side, so we can easily edit the task before restarting it.
/// It's also necessary to get all failed tasks, in case the user specified the `--failed` flag.
#[allow(clippy::too_many_arguments)]
pub async fn restart(
    client: &mut Client,
    settings: Settings,
    task_ids: Vec<usize>,
    all: bool,
    failed: bool,
    group: Option<String>,
    start_immediately: bool,
    stashed: bool,
    in_place: bool,
    not_in_place: bool,
    edit: bool,
) -> Result<()> {
    // `not_in_place` superseeds both other configs
    let in_place = (settings.client.restart_in_place || in_place) && !not_in_place;

    let new_status = if stashed {
        TaskStatus::Stashed { enqueue_at: None }
    } else {
        TaskStatus::Queued {
            enqueued_at: Local::now(),
        }
    };

    let state = get_state(client).await?;

    // Filter to get done tasks
    let done_filter = |task: &Task| task.is_done();

    // Determine which tasks to restart based on the combination of flags
    let filtered_tasks = if all || group.is_some() {
        // --all or --group is specified, get all finished tasks
        let filtered_tasks = if let Some(group_name) = &group {
            // --group: specific group's finished tasks
            state.filter_tasks_of_group(done_filter, group_name)
        } else {
            // --all: all groups' finished tasks
            state.filter_tasks(done_filter, None)
        };

        // Apply --failed filter if specified
        let filtered = if failed {
            // --failed: only failed tasks
            filtered_tasks
                .matching_ids
                .into_iter()
                .filter(|task_id| {
                    let task = state.tasks.get(task_id).unwrap();
                    !matches!(
                        task.status,
                        TaskStatus::Done {
                            result: TaskResult::Success,
                            ..
                        }
                    )
                })
                .collect()
        } else {
            // No --failed: all finished tasks
            filtered_tasks.matching_ids
        };

        FilteredTasks {
            matching_ids: filtered,
            ..Default::default()
        }
    } else if task_ids.is_empty() {
        bail!(
            "Please provide the ids of the tasks you want to restart, use --all, or use --group."
        );
    } else {
        // Specific task_ids provided
        state.filter_tasks(done_filter, Some(task_ids))
    };

    // Build a RestartMessage, if the tasks should be replaced instead of creating a copy of the
    // original task. This is only important, if replace is `True`.
    let mut restart_message = RestartRequest {
        tasks: Vec::new(),
        stashed,
        start_immediately,
    };

    // Get all tasks that should be restarted.
    let mut tasks: BTreeMap<usize, Task> = filtered_tasks
        .matching_ids
        .iter()
        .map(|task_id| (*task_id, state.tasks.get(task_id).unwrap().clone()))
        .collect();

    // If the tasks should be edited, edit them in one go.
    if edit {
        let mut editable_tasks: Vec<EditableTask> =
            tasks.values().map(EditableTask::from).collect();
        editable_tasks = edit_tasks(&settings, editable_tasks)?;

        // Now merge the edited properties back into the tasks.
        for edited in editable_tasks {
            let task = tasks.get_mut(&edited.id).context(format!(
                "Found unexpected task id during editing: {}",
                edited.id
            ))?;
            edited.into_task(task);
        }
    }

    // Go through all restartable commands we found and process them.
    for (_, mut task) in tasks {
        task.status = new_status.clone();

        // Add the tasks to the singular message, if we want to restart the tasks in-place.
        // And continue with the next task. The message will then be sent after the for loop.
        if in_place {
            restart_message.tasks.push(TaskToRestart {
                task_id: task.id,
                original_command: task.original_command,
                path: task.path,
                label: task.label,
                priority: task.priority,
            });

            continue;
        }

        // In case we don't do in-place restarts, we have to add a new task.
        // Create an request to send the task to the daemon from the updated info and the old
        // task.
        let add_task_message = AddRequest {
            command: task.original_command,
            path: task.path,
            envs: task.envs.clone(),
            start_immediately,
            stashed,
            group: task.group.clone(),
            enqueue_at: None,
            dependencies: Vec::new(),
            priority: Some(task.priority),
            label: task.label,
        };

        // Send the cloned task to the daemon and abort on any failure messages.
        client.send_request(add_task_message).await?;
        if let Response::Failure(message) = client.receive_response().await? {
            bail!(message);
        };
    }

    // Send the singular in-place restart message to the daemon.
    if in_place {
        client.send_request(restart_message).await?;
        if let Response::Failure(message) = client.receive_response().await? {
            bail!(message);
        };
    }

    if !filtered_tasks.matching_ids.is_empty() {
        println!("Restarted tasks: {:?}", filtered_tasks.matching_ids);
    }
    if !filtered_tasks.non_matching_ids.is_empty() {
        eprintln!(
            "Couldn't restart tasks: {:?}",
            filtered_tasks.non_matching_ids
        );
    }

    Ok(())
}
