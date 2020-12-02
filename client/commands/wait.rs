use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use chrono::Local;

use pueue::protocol::Socket;
use pueue::task::{Task, TaskStatus};

use crate::commands::get_state;

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
    socket: &mut Socket,
    task_ids: &Option<Vec<usize>>,
    group: &Option<String>,
    all: bool,
    quiet: bool,
) -> Result<()> {
    let mut first_run = true;
    // Create a list of tracked tasks.
    // This way we can track any status changes and if any new tasks are added.
    let mut watched_tasks: HashMap<usize, TaskStatus> = HashMap::new();

    loop {
        let state = get_state(socket).await?;

        let tasks: Vec<Task> = if all {
            // Get all tasks
            state.tasks.iter().map(|(_, task)| task.clone()).collect()
        } else if let Some(group) = group {
            // Get all tasks of a specific group
            state
                .tasks
                .iter()
                .filter(|(_, task)| task.group == Some(group.clone()))
                .map(|(_, task)| task.clone())
                .collect()
        } else if let Some(ids) = task_ids {
            // Get all tasks of a specific group
            state
                .tasks
                .iter()
                .filter(|(id, _)| ids.contains(id))
                .map(|(_, task)| task.clone())
                .collect()
        } else {
            // Get all tasks of the default queue by default
            state
                .tasks
                .iter()
                .filter(|(_, task)| task.group == None)
                .map(|(_, task)| task.clone())
                .collect()
        };

        // Get current time for log output
        let current_time = Local::now().format("%H:%M:%S");

        // Iterate over all matching tasks
        for task in tasks.iter() {
            // Check if we already know this task or if it is new.
            let previous_status = match watched_tasks.get(&task.id) {
                None => {
                    // Add any unknown tasks to our watchlist
                    // Don't log anything if this is the first run
                    if !quiet && !first_run {
                        println!(
                            "{} - New task {} with status {}",
                            current_time, task.id, task.status
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
                println!(
                    "{} - Task {} changed from {} to {}",
                    current_time, task.id, previous_status, task.status
                );
            }
        }

        // We can stop waiting, if every task is on `Done`
        // Always check the actual task list instead of the watched_tasks list.
        // Otherwise we get locked if tasks get removed.
        let all_finished = tasks.iter().all(|task| task.status == TaskStatus::Done);

        if all_finished {
            break;
        }

        // Sleep for a few milliseconds. We don't want to hurt the CPU.
        let timeout = Duration::from_millis(2000);
        // Don't use recv_timeout for now, until this bug get's fixed.
        // https://github.com/rust-lang/rust/issues/39364
        //match self.receiver.recv_timeout(timeout) {
        sleep(timeout);
        first_run = false;
    }

    Ok(())
}
