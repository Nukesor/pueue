use std::path::PathBuf;

use anyhow::Context;
use anyhow::{bail, Result};

use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::commands::edit::edit_line_wrapper;
use crate::commands::get_state;

/// When restarting tasks, the remote state is queried and a [message::AddMessage]
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
) -> Result<()> {
    let new_status = if stashed {
        TaskStatus::Stashed { enqueue_at: None }
    } else {
        TaskStatus::Queued
    };

    let state = get_state(stream).await?;

    // Filter to get done tasks
    let done_filter = |task: &Task| task.is_done();

    let (matching, mismatching) = if all_failed || failed_in_group.is_some() {
        // Either all failed tasks or all failed tasks of a specific group need to be restarted.

        // First we have to get all finished tasks (Done)
        let (matching, _) = if let Some(group) = failed_in_group {
            state.filter_tasks_of_group(done_filter, &group)
        } else {
            state.filter_tasks(done_filter, None)
        };

        // now pick the failed tasks
        let failed = matching
            .into_iter()
            .filter(|task_id| {
                let task = state.tasks.get(task_id).unwrap();
                !matches!(task.status, TaskStatus::Done(TaskResult::Success))
            })
            .collect();

        // We return an empty vec for the mismatching tasks, since there shouldn't be any.
        // Any User provided ids are ignored in this mode.
        (failed, Vec::new())
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
    for task_id in &matching {
        let task = state.tasks.get(task_id).unwrap();
        let mut new_task = Task::from_task(task);
        new_task.status = new_status.clone();

        // Path and command can be edited, if the use specified the -e or -p flag.
        let mut command = None;
        let mut path = None;

        // Update the command if requested.
        if edit_command {
            command = Some(edit_line_wrapper(stream, *task_id, &task.command).await?);
        };

        // Update the path if requested.
        if edit_path {
            let str_path = task
                .path
                .to_str()
                .context("Failed to convert task path to string")?;
            let changed_path = edit_line_wrapper(stream, *task_id, str_path).await?;
            path = Some(PathBuf::from(changed_path));
        }

        // Add the tasks to the singular message, if we want to restart the tasks in-place.
        // And continue with the next task. The message will then be sent after the for loop.
        if in_place {
            restart_message.tasks.push(TaskToRestart {
                task_id: *task_id,
                command,
                path,
            });

            continue;
        }

        // In case we don't do in-place restarts, we have to add a new task.
        // Create a AddMessage to send the task to the daemon from the updated info and the old task.
        let add_task_message = Message::Add(AddMessage {
            command: command.unwrap_or_else(|| task.command.clone()),
            path: path.unwrap_or_else(|| task.path.clone()),
            envs: task.envs.clone(),
            start_immediately,
            stashed,
            group: task.group.clone(),
            enqueue_at: None,
            dependencies: Vec::new(),
            label: task.label.clone(),
            print_task_id: false,
        });

        // Send the cloned task to the daemon and abort on any failure messages.
        send_message(add_task_message, stream).await?;
        if let Message::Failure(message) = receive_message(stream).await? {
            bail!(message);
        };
    }

    // Send the singular in-place restart message to the daemon.
    if in_place {
        send_message(Message::Restart(restart_message), stream).await?;
        if let Message::Failure(message) = receive_message(stream).await? {
            bail!(message);
        };
    }

    if !matching.is_empty() {
        println!("Restarted tasks: {matching:?}");
    }
    if !mismatching.is_empty() {
        println!("Couldn't restart tasks: {mismatching:?}");
    }

    Ok(())
}
