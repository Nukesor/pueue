use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use chrono::Local;
use crossterm::style::{Attribute, Color};

use pueue::network::protocol::GenericStream;
use pueue::task::{Task, TaskResult, TaskStatus};

use crate::commands::get_state;
use crate::output_helper::style_text;

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
    task_ids: &Option<Vec<usize>>,
    group: &str,
    all: bool,
    quiet: bool,
) -> Result<()> {
    let mut first_run = true;
    // Create a list of tracked tasks.
    // This way we can track any status changes and if any new tasks are added.
    let mut watched_tasks: HashMap<usize, TaskStatus> = HashMap::new();

    loop {
        let state = get_state(stream).await?;

        let tasks: Vec<Task> = if all {
            // Get all tasks
            state.tasks.iter().map(|(_, task)| task.clone()).collect()
        } else if let Some(ids) = task_ids {
            // Get all tasks of a specific group
            state
                .tasks
                .iter()
                .filter(|(id, _)| ids.contains(id))
                .map(|(_, task)| task.clone())
                .collect()
        } else {
            // Get all tasks of a specific group
            state
                .tasks
                .iter()
                .filter(|(_, task)| task.group.eq(group))
                .map(|(_, task)| task.clone())
                .collect()
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
                        let color = get_color_for_status(&task.status);
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
                log_status_change(&current_time, previous_status, &task);
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

fn log_status_change(current_time: &str, previous_status: TaskStatus, task: &Task) {
    // Finishing tasks get some special handling
    if task.status == TaskStatus::Done {
        let text = match task.result {
            Some(TaskResult::Success) => {
                format!(
                    "Task {} succeeded with {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("0", Some(Color::Green), None)
                )
            }
            Some(TaskResult::DependencyFailed) => {
                format!(
                    "Task {} failed due to {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("failed dependencies", Some(Color::Red), None)
                )
            }

            Some(TaskResult::FailedToSpawn(_)) => {
                format!(
                    "Task {} {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("failed to spawn", Some(Color::Red), None)
                )
            }
            Some(TaskResult::Failed(exit_code)) => {
                format!(
                    "Task {} failed with {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text(exit_code, Some(Color::Red), Some(Attribute::Bold))
                )
            }
            Some(TaskResult::Killed) => {
                format!(
                    "Task {} has been {}",
                    style_text(task.id, None, Some(Attribute::Bold)),
                    style_text("killed", Some(Color::Red), None)
                )
            }
            None => panic!("Got a 'Done' task without a task result. Please report this bug."),
        };
        println!("{} - {}", current_time, text);

        return;
    }
    let new_status_color = get_color_for_status(&task.status);
    let previous_status_color = get_color_for_status(&previous_status);

    println!(
        "{} - Task {} changed from {} to {}",
        current_time,
        style_text(task.id, None, Some(Attribute::Bold)),
        style_text(previous_status, Some(previous_status_color), None),
        style_text(&task.status, Some(new_status_color), None),
    );
}

fn get_color_for_status(task_status: &TaskStatus) -> Color {
    match task_status {
        TaskStatus::Running | TaskStatus::Done => Color::Green,
        TaskStatus::Paused | TaskStatus::Locked => Color::White,
        _ => Color::Yellow,
    }
}
