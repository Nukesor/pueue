use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use chrono::Local;
use crossterm::style::{Attribute, Color};

use pueue_lib::network::protocol::GenericStream;
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::display::helper::style_text;
use crate::{commands::get_state, display::colors::Colors};

/// Wait until tasks are done.
/// Tasks can be specified by:
/// - Default queue (no parameter given)
/// - Group
/// - A list of task ids
/// - All tasks (`all == true`)
///
/// By default, this will log status changes on tasks.
/// Pass `quiet == true` to supress any logging.
pub async fn wait(
    stream: &mut GenericStream,
    task_ids: &[usize],
    group: &str,
    all: bool,
    quiet: bool,
    colors: &Colors,
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
                        let color = get_color_for_status(&task.status, colors);
                        let task_id = style_text(task.id, None, Some(Attribute::Bold));
                        let status = style_text(&task.status, Some(color), None);

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
                log_status_change(&current_time, previous_status, task, colors);
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
        sleep(Duration::from_millis(2000));
        first_run = false;
    }

    Ok(())
}

fn log_status_change(
    current_time: &str,
    previous_status: TaskStatus,
    task: &Task,
    colors: &Colors,
) {
    // Finishing tasks get some special handling
    if let TaskStatus::Done(result) = &task.status {
        let text = match result {
            TaskResult::Success => {
                let task_id = style_text(task.id, None, Some(Attribute::Bold));
                let status = style_text("0", Some(colors.green()), None);
                format!("Task {task_id} succeeded with {status}")
            }
            TaskResult::DependencyFailed => {
                let task_id = style_text(task.id, None, Some(Attribute::Bold));
                let status = style_text("failed dependencies", Some(colors.red()), None);
                format!("Task {task_id} failed due to {status}")
            }

            TaskResult::FailedToSpawn(_) => {
                let task_id = style_text(task.id, None, Some(Attribute::Bold));
                let status = style_text("failed to spawn", Some(colors.red()), None);
                format!("Task {task_id} {status}")
            }
            TaskResult::Failed(exit_code) => {
                let task_id = style_text(task.id, None, Some(Attribute::Bold));
                let status = style_text(exit_code, Some(colors.red()), Some(Attribute::Bold));
                format!("Task {task_id} failed with {status}")
            }
            TaskResult::Errored => {
                let task_id = style_text(task.id, None, Some(Attribute::Bold));
                let status = style_text("IO error", Some(colors.red()), Some(Attribute::Bold));
                format!("Task {task_id} experienced an {status}.")
            }
            TaskResult::Killed => {
                let task_id = style_text(task.id, None, Some(Attribute::Bold));
                let status = style_text("killed", Some(colors.red()), None);
                format!("Task {task_id} has been {status}")
            }
        };
        println!("{current_time} - {text}");

        return;
    }
    let new_status_color = get_color_for_status(&task.status, colors);
    let previous_status_color = get_color_for_status(&previous_status, colors);

    let task_id = style_text(task.id, None, Some(Attribute::Bold));
    let previous_status = style_text(previous_status, Some(previous_status_color), None);
    let new_status = style_text(&task.status, Some(new_status_color), None);
    println!("{current_time} - Task {task_id} changed from {previous_status} to {new_status}",);
}

fn get_color_for_status(task_status: &TaskStatus, colors: &Colors) -> Color {
    match task_status {
        TaskStatus::Running | TaskStatus::Done(_) => colors.green(),
        TaskStatus::Paused | TaskStatus::Locked => colors.white(),
        _ => colors.white(),
    }
}
