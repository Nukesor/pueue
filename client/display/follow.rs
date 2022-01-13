use std::io;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use pueue_lib::log::{get_log_file_handles, get_log_paths, seek_to_last_lines};

/// Follow the log ouput of running task.
///
/// If no task is specified, this will check for the following cases:
///
/// - No running task: Print an error that there are no running tasks
/// - Single running task: Follow the output of that task
/// - Multiple running tasks: Print out the list of possible tasks to follow.
pub fn follow_local_task_logs(
    pueue_directory: &Path,
    task_id: usize,
    stderr: bool,
    lines: Option<usize>,
) {
    let (stdout_handle, stderr_handle) = match get_log_file_handles(task_id, pueue_directory) {
        Ok((stdout, stderr)) => (stdout, stderr),
        Err(err) => {
            println!("Failed to get log file handles: {err}");
            return;
        }
    };
    let mut handle = if stderr { stderr_handle } else { stdout_handle };

    let (out_path, err_path) = get_log_paths(task_id, pueue_directory);
    let handle_path = if stderr { err_path } else { out_path };

    // Stdout handler to directly write log file output to io::stdout
    // without having to load anything into memory.
    let mut stdout = io::stdout();

    // If lines is passed as an option, seek the output file handle to the start of
    // the line corresponding to the `lines` number of lines from the end of the file.
    // The loop following this section will copy those lines to stdout
    if let Some(lines) = lines {
        if let Err(err) = seek_to_last_lines(&mut handle, lines) {
            println!("Error seeking to last lines from log: {err}");
        }
    }
    loop {
        // Check whether the file still exists. Exit if it doesn't.
        if !handle_path.exists() {
            println!("File has gone away. Did somebody remove the task?");
            return;
        }
        // Read the next chunk of text from the last position.
        if let Err(err) = io::copy(&mut handle, &mut stdout) {
            println!("Error while reading file: {err}");
            return;
        };
        let timeout = Duration::from_millis(100);
        sleep(timeout);
    }
}
