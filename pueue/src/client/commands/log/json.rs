use std::{
    collections::{BTreeMap, HashMap},
    io::Read,
};

use pueue_lib::{
    log::{get_log_file_handle, read_last_lines},
    message::TaskLogResponse,
    settings::Settings,
    task::Task,
};
use serde::{Deserialize, Serialize};
use snap::read::FrameDecoder;

/// This is the output struct used for
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskLog {
    pub task: Task,
    pub full_output_bytes: u64,
    pub output: String,
}

/// Print some log output in JSON serialized form.
///
/// If the log isn't read from the disk but rather received from the daemon, we have to
/// convert the received [TaskLogResponse] into a proper JSON serializable format.
/// Output in [TaskLogResponse], is usually compressed, so we need to decompress it first.
pub fn print_log_json(
    task_log_messages: BTreeMap<usize, TaskLogResponse>,
    settings: &Settings,
    lines: Option<usize>,
) {
    let mut tasks: BTreeMap<usize, Task> = BTreeMap::new();
    let mut task_log: BTreeMap<usize, (String, u64)> = BTreeMap::new();
    for (id, message) in task_log_messages {
        tasks.insert(id, message.task);

        if settings.client.read_local_logs {
            let (output, full_output_bytes) = get_local_log(settings, id, lines);
            task_log.insert(id, (output, full_output_bytes));
        } else {
            let output = get_remote_log(message.output);
            task_log.insert(id, (output, message.full_output_bytes));
        }
    }

    // Now assemble the final struct that will be returned
    let mut json = BTreeMap::new();
    for (id, mut task) in tasks {
        let (id, (output, full_output_bytes)) = task_log.remove_entry(&id).unwrap();

        task.envs = HashMap::new();
        json.insert(
            id,
            TaskLog {
                task,
                full_output_bytes,
                output,
            },
        );
    }

    println!("{}", serde_json::to_string(&json).unwrap());
}

/// Read logs directly from local files for a specific task.
fn get_local_log(settings: &Settings, id: usize, lines: Option<usize>) -> (String, u64) {
    let mut file = match get_log_file_handle(id, &settings.shared.pueue_directory()) {
        Ok(file) => file,
        Err(err) => {
            return (
                format!("(Pueue error) Failed to get log file handle: {err}"),
                0,
            );
        }
    };
    let full_output_bytes = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            return (
                format!("(Pueue error) Failed to get local log metadata: {error:?}"),
                0,
            );
        }
    };

    // Only return the last few lines.
    if let Some(lines) = lines {
        return (read_last_lines(&mut file, lines), full_output_bytes);
    }

    // Read the whole local log output.
    let mut output = String::new();
    if let Err(error) = file.read_to_string(&mut output) {
        return (
            format!("(Pueue error) Failed to read local log output file: {error:?}"),
            full_output_bytes,
        );
    };

    (output, full_output_bytes)
}

/// Read logs from from compressed remote logs.
/// If logs don't exist, an empty string will be returned.
fn get_remote_log(output_bytes: Option<Vec<u8>>) -> String {
    let Some(bytes) = output_bytes else {
        return String::new();
    };

    let mut decoder = FrameDecoder::new(&bytes[..]);
    let mut output = String::new();
    if let Err(error) = decoder.read_to_string(&mut output) {
        return format!("(Pueue error) Failed to decompress remote log output: {error:?}");
    }

    output
}
