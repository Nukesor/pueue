use std::io;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use tokio::time::sleep;

use pueue_lib::{
    log::{get_log_file_handle, get_log_path, seek_to_last_lines},
    network::protocol::GenericStream,
};

use crate::commands::get_state;

/// Follow the log ouput of running task.
///
/// If no task is specified, this will check for the following cases:
///
/// - No running task: Print an error that there are no running tasks.
/// - Single running task: Follow the output of that task.
/// - Multiple running tasks: Print out the list of possible tasks to follow.
pub async fn follow_local_task_logs(
    stream: &mut GenericStream,
    pueue_directory: &Path,
    task_id: usize,
    lines: Option<usize>,
) -> Result<()> {
    let mut handle = match get_log_file_handle(task_id, pueue_directory) {
        Ok(stdout) => stdout,
        Err(err) => {
            println!("Failed to get log file handles: {err}");
            return Ok(());
        }
    };
    let path = get_log_path(task_id, pueue_directory);

    // Stdout handle to directly stream log file output to `io::stdout`.
    // This prevents us from allocating any large amounts of memory.
    let mut stdout = io::stdout();

    // If `lines` is passed as an option, we only want to show the last `X` lines.
    // To achieve this, we seek the file handle to the start of the `Xth` line
    // from the end of the file.
    // The loop following this section will then only copy those last lines to stdout.
    if let Some(lines) = lines {
        if let Err(err) = seek_to_last_lines(&mut handle, lines) {
            println!("Error seeking to last lines from log: {err}");
        }
    }

    // We check in regular intervals whether the task finished.
    // This is something we don't want to do in every loop, as we have to communicate with
    // the daemon. That's why we only do it now and then.
    let task_check_interval = 2000;
    let mut last_check = 2000;

    let log_check_interval = 100;
    loop {
        // Check whether the file still exists. Exit if it doesn't.
        if !path.exists() {
            println!("Log file has gone away. Has the task been removed?");
            return Ok(());
        }
        // Read the next chunk of text from the last position.
        if let Err(err) = io::copy(&mut handle, &mut stdout) {
            println!("Error while reading file: {err}");
            return Ok(());
        };

        // Do a check at the very beginning and every `task_check_interval`, whether the task:
        // 1. Still exist
        // 2. Is still running
        //
        // In case it's not, exit.
        if (last_check % task_check_interval) == 0 {
            let state = get_state(stream).await?;
            let Some(task) = state.tasks.get(&task_id) else {
                println!("Pueue: The followed task has been removed.",);
                return Ok(());
            };

            // The task is done.
            if !task.is_running() {
                return Ok(());
            }
        }

        last_check += log_check_interval;
        let timeout = Duration::from_millis(log_check_interval);
        sleep(timeout).await;
    }
}
