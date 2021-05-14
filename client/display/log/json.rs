use std::collections::BTreeMap;
use std::io::Read;

use serde_derive::{Deserialize, Serialize};
use snap::read::FrameDecoder;

use pueue_lib::log::{get_log_file_handles, read_last_lines};
use pueue_lib::network::message::TaskLogMessage;
use pueue_lib::settings::Settings;
use pueue_lib::task::Task;

/// This is the output struct used for
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskLog {
    pub task: Task,
    pub stdout: String,
    pub stderr: String,
}

pub fn print_log_json(
    task_log_messages: BTreeMap<usize, TaskLogMessage>,
    settings: &Settings,
    lines: Option<usize>,
) {
    let mut tasks: BTreeMap<usize, Task> = BTreeMap::new();
    let mut task_log: BTreeMap<usize, (String, String)> = BTreeMap::new();
    // Convert the TaskLogMessage into a proper JSON serializable format.
    // Output in TaskLogMessages, if it exists, is compressed.
    // We need to decompress and convert to normal strings.
    for (id, message) in task_log_messages {
        tasks.insert(id, message.task);

        if settings.client.read_local_logs {
            let output = get_local_logs(settings, id, lines);
            task_log.insert(id, output);
        } else {
            let output = get_remote_logs(message.stdout, message.stderr);
            task_log.insert(id, output);
        }
    }

    // Now assemble the final struct that will be returned
    let mut json = BTreeMap::new();
    for (id, task) in tasks {
        let (id, (stdout, stderr)) = task_log.remove_entry(&id).unwrap();

        json.insert(
            id,
            TaskLog {
                task,
                stdout,
                stderr,
            },
        );
    }

    println!("{}", serde_json::to_string(&json).unwrap());
}

/// Read logs directly from local files for a specific task.
fn get_local_logs(settings: &Settings, id: usize, lines: Option<usize>) -> (String, String) {
    let (mut stdout_file, mut stderr_file) =
        match get_log_file_handles(id, &settings.shared.pueue_directory()) {
            Ok((stdout, stderr)) => (stdout, stderr),
            Err(err) => {
                let error = format!("(Pueue error) Failed to get log file handles: {}", err);
                return (String::new(), error);
            }
        };

    let stdout = if let Some(lines) = lines {
        read_last_lines(&mut stdout_file, lines)
    } else {
        let mut stdout = String::new();
        if let Err(error) = stdout_file.read_to_string(&mut stdout) {
            stdout.push_str(&format!(
                "(Pueue error) Failed to read local log output file: {:?}",
                error
            ))
        };

        stdout
    };

    let stderr = if let Some(lines) = lines {
        read_last_lines(&mut stderr_file, lines)
    } else {
        let mut stderr = String::new();
        if let Err(error) = stderr_file.read_to_string(&mut stderr) {
            stderr.push_str(&format!(
                "(Pueue error) Failed to read local log output file: {:?}",
                error
            ))
        };

        stderr
    };

    (stdout, stderr)
}

/// Read logs from from compressed remote logs.
/// If logs don't exist, an empty string will be returned.
fn get_remote_logs(
    stdout_bytes: Option<Vec<u8>>,
    stderr_bytes: Option<Vec<u8>>,
) -> (String, String) {
    let stdout = if let Some(bytes) = stdout_bytes {
        let mut decoder = FrameDecoder::new(&bytes[..]);
        let mut stdout = String::new();
        if let Err(error) = decoder.read_to_string(&mut stdout) {
            stdout.push_str(&format!(
                "(Pueue error) Failed to decompress remote log output: {:?}",
                error
            ))
        }
        stdout
    } else {
        String::new()
    };

    let stderr = if let Some(bytes) = stderr_bytes {
        let mut decoder = FrameDecoder::new(&bytes[..]);
        let mut stderr = String::new();
        if let Err(error) = decoder.read_to_string(&mut stderr) {
            stderr.push_str(&format!(
                "(Pueue error) Failed to decompress remote log output: {:?}",
                error
            ))
        }

        stderr
    } else {
        String::new()
    };

    (stdout, stderr)
}
