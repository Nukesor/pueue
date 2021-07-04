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
                println!("No tasks found for group {}", group);
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
                    // Don't log anything if this is the first run
                    if !quiet && !first_run {
                        let color = get_color_for_status(&task.status, colors);
                        println!(
                            "{} - New task {} with status {}",
                            current_time,
                            style_text(task.id, None, Some(Attribute::Bold)),
                            style_text(&task.status, Some(color), None),
                        );
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
        let all_finished = tasks.iter().all(|task| task.status == TaskStatus::Done);

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
    if task.status == TaskStatus::Done {
        let text = match task.result {
            Some(TaskResult::Success) => {
                format!(
                    "Task {} succeeded with {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("0", Some(colors.green()), None)
                )
            }
            Some(TaskResult::DependencyFailed) => {
                format!(
                    "Task {} failed due to {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("failed dependencies", Some(colors.red()), None)
                )
            }

            Some(TaskResult::FailedToSpawn(_)) => {
                format!(
                    "Task {} {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("failed to spawn", Some(colors.red()), None)
                )
            }
            Some(TaskResult::Failed(exit_code)) => {
                format!(
                    "Task {} failed with {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text(exit_code, Some(colors.red()), Some(Attribute::Bold))
                )
            }
            Some(TaskResult::Errored) => {
                format!(
                    "Task {} experienced an {}.",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("IO error", Some(colors.red()), Some(Attribute::Bold))
                )
            }
            Some(TaskResult::Killed) => {
                format!(
                    "Task {} has been {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("killed", Some(colors.red()), None)
                )
            }
            None => panic!("Got a 'Done' task without a task result. Please report this bug."),
        };
        println!("{} - {}", current_time, text);

        return;
    }
    let new_status_color = get_color_for_status(&task.status, colors);
    let previous_status_color = get_color_for_status(&previous_status, colors);

    println!(
        "{} - Task {} changed from {} to {}",
        current_time,
        style_text(task.id, None, Some(Attribute::Bold)),
        style_text(previous_status, Some(previous_status_color), None),
        style_text(&task.status, Some(new_status_color), None),
    );
}

fn get_color_for_status(task_status: &TaskStatus, colors: &Colors) -> Color {
    match task_status {
        TaskStatus::Running | TaskStatus::Done => colors.green(),
        TaskStatus::Paused | TaskStatus::Locked => colors.white(),
        _ => colors.white(),
    }
}
