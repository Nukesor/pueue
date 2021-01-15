use std::collections::BTreeMap;

use pueue::log::read_and_compress_log_files;
use pueue::network::message::*;
use pueue::state::SharedState;

/// Invoked when calling `pueue log`.
/// Return the current state and the stdou/stderr of all tasks to the client.
pub fn get_log(message: LogRequestMessage, state: &SharedState) -> Message {
    let state = state.lock().unwrap().clone();
    // Return all logs, if no specific task id is specified.
    let task_ids = if message.task_ids.is_empty() {
        state.tasks.keys().cloned().collect()
    } else {
        message.task_ids
    };

    let mut tasks = BTreeMap::new();
    for task_id in task_ids.iter() {
        if let Some(task) = state.tasks.get(task_id) {
            // We send log output and the task at the same time.
            // This isn't as efficient as sending the raw compressed data directly,
            // but it's a lot more convenient for now.
            let (stdout, stderr) = if message.send_logs {
                match read_and_compress_log_files(*task_id, &state.settings.shared.pueue_directory)
                {
                    Ok((stdout, stderr)) => (Some(stdout), Some(stderr)),
                    Err(err) => {
                        // Fail early if there's some problem with getting the log output
                        return create_failure_message(format!(
                            "Failed reading process output file: {:?}",
                            err
                        ));
                    }
                }
            } else {
                (None, None)
            };

            let task_log = TaskLogMessage {
                task: task.clone(),
                stdout,
                stderr,
            };
            tasks.insert(*task_id, task_log);
        }
    }
    Message::LogResponse(tasks)
}
