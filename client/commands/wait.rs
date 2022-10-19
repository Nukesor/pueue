use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use chrono::Local;
use crossterm::style::{Attribute, Color};
use tokio::time::sleep;

use pueue_lib::network::protocol::GenericStream;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::{commands::get_state, display::OutputStyle};

/// Wait until tasks are done.
/// Tasks can be specified by:
/// - Default queue (no parameter given)
/// - Group
/// - A list of task ids
/// - All tasks (`all == true`)
///
/// By default, this will output status changes of tasks to `stdout`.
/// Pass `quiet == true` to supress any logging.
pub async fn wait(
    stream: &mut GenericStream,
    task_ids: &[usize],
    group: &str,
    all: bool,
    quiet: bool,
    style: &OutputStyle,
) -> Result<()> {
    let mut first_run = true;
    // Create a list of tracked tasks.
    // This way we can track any status changes and if any new tasks are added.
    let mut watched_tasks: HashMap<usize, TaskStatus> = HashMap::new();

    loop {
        let state = get_state(stream).await?;

        let tasks: Vec<Task> = if !task_ids.is_empty() {
            // Get all tasks of a specific group
            state
                .tasks
                .iter()
                .filter(|(id, _)| task_ids.contains(id))
                .map(|(_, task)| task.clone())
                .collect()
        } else if all {
            // Get all tasks
            state.tasks.iter().map(|(_, task)| task.clone()).collect()
        } else {
            // Get all tasks of a specific group
            let tasks = state
                .tasks
                .iter()
                .filter(|(_, task)| task.group.eq(group))
                .map(|(_, task)| task.clone())
                .collect::<Vec<Task>>();

            if tasks.is_empty() {
                println!("No tasks found for group {group}");
                return Ok(());
            }

            tasks
        };

        // Get current time for log output
        let current_time = Local::now().format("%H:%M:%S").to_string();

        // Iterate over all matching tasks
        for task in tasks.iter() {
            // Check if we already know this task or if it is new.
            let previous_status = match watched_tasks.get(&task.id) {
                None => {
                    // Add any unknown tasks to our watchlist
                    if !quiet {
                        let color = get_color_for_status(&task.status);
                        let task_id = style.style_text(task.id, None, Some(Attribute::Bold));
                        let status = style.style_text(&task.status, Some(color), None);

                        if !first_run {
                            // Don't log non-active tasks in the initial loop.
                            println!("{current_time} - New task {task_id} with status {status}",);
                        } else if task.is_running() {
                            // Show currently running tasks for better user feedback.
                            println!(
                                "{current_time} - Found active Task {task_id} with status {status}",
                            );
                        }
                    }

                    watched_tasks.insert(task.id, task.status.clone());

                    continue;
                }
                Some(previous_status) => {
                    if previous_status == &task.status {
                        continue;
                    }
                    previous_status.clone()
                }
            };

            // Update the (previous) task status and log any changes
            watched_tasks.insert(task.id, task.status.clone());
            if !quiet {
                log_status_change(&current_time, previous_status, task, style);
            }
        }

        // We can stop waiting, if every task is on `Done`
        // Always check the actual task list instead of the watched_tasks list.
        // Otherwise we get locked if tasks get removed.
        let all_finished = tasks
            .iter()
            .all(|task| matches!(task.status, TaskStatus::Done(_)));

        if all_finished {
            break;
        }

        // Sleep for a few seconds. We don't want to hurt the CPU.
        // However, we allow faster polling when in a test environment.
        let mut sleep_time = 2000;
        if std::env::var("PUEUED_TEST_ENV_VARIABLE").is_ok() {
            sleep_time = 250;
        }
        sleep(Duration::from_millis(sleep_time)).await;
        first_run = false;
    }

    Ok(())
}

fn log_status_change(
    current_time: &str,
    previous_status: TaskStatus,
    task: &Task,
    style: &OutputStyle,
) {
    let task_id = style.style_text(task.id, None, Some(Attribute::Bold));

    // Check if the task has finished.
    // In case it has, show the task's result in human-readable form.
    // Color some parts of the output depending on the task's outcome.
    if let TaskStatus::Done(result) = &task.status {
        let text = match result {
            TaskResult::Success => {
                let status = style.style_text("0", Some(Color::Green), None);
                format!("Task {task_id} succeeded with {status}")
            }
            TaskResult::DependencyFailed => {
                let status = style.style_text("failed dependencies", Some(Color::Red), None);
                format!("Task {task_id} failed due to {status}")
            }

            TaskResult::FailedToSpawn(_) => {
                let status = style.style_text("failed to spawn", Some(Color::Red), None);
                format!("Task {task_id} {status}")
            }
            TaskResult::Failed(exit_code) => {
                let status = style.style_text(exit_code, Some(Color::Red), Some(Attribute::Bold));
                format!("Task {task_id} failed with {status}")
            }
            TaskResult::Errored => {
                let status = style.style_text("IO error", Some(Color::Red), Some(Attribute::Bold));
                format!("Task {task_id} experienced an {status}.")
            }
            TaskResult::Killed => {
                let status = style.style_text("killed", Some(Color::Red), None);
                format!("Task {task_id} has been {status}")
            }
        };
        println!("{current_time} - {text}");

        return;
    }

    // The task didn't finish yet, but changed it's state (e.g. from `Queued` to `Running`).
    // Inform the user about this change.
    let new_status_color = get_color_for_status(&task.status);
    let previous_status_color = get_color_for_status(&previous_status);

    let previous_status = style.style_text(previous_status, Some(previous_status_color), None);
    let new_status = style.style_text(&task.status, Some(new_status_color), None);
    println!("{current_time} - Task {task_id} changed from {previous_status} to {new_status}",);
}

fn get_color_for_status(task_status: &TaskStatus) -> Color {
    match task_status {
        TaskStatus::Running | TaskStatus::Done(_) => Color::Green,
        TaskStatus::Paused | TaskStatus::Locked => Color::White,
        _ => Color::White,
    }
}
