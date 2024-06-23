use std::collections::BTreeMap;

use pueue_lib::failure_msg;
use pueue_lib::log::read_and_compress_log_file;
use pueue_lib::network::message::*;
use pueue_lib::settings::Settings;
use pueue_lib::state::SharedState;

/// Invoked when calling `pueue log`.
/// Return tasks and their output to the client.
pub fn get_log(settings: &Settings, state: &SharedState, message: LogRequestMessage) -> Message {
    let state = { state.lock().unwrap().clone() };

    let task_ids = match message.tasks {
        TaskSelection::All => state.tasks.keys().cloned().collect(),
        TaskSelection::TaskIds(task_ids) => task_ids,
        TaskSelection::Group(group) => state.task_ids_in_group(&group),
    };

    let mut tasks = BTreeMap::new();
    for task_id in task_ids.iter() {
        if let Some(task) = state.tasks.get(task_id) {
            // We send log output and the task at the same time.
            // This isn't as efficient as sending the raw compressed data directly,
            // but it's a lot more convenient for now.
            let (output, output_complete) = if message.send_logs {
                match read_and_compress_log_file(
                    *task_id,
                    &settings.shared.pueue_directory(),
                    message.lines,
                ) {
                    Ok((output, output_complete)) => (Some(output), output_complete),
                    Err(err) => {
                        // Fail early if there's some problem with getting the log output
                        return failure_msg!("Failed reading process output file: {err:?}");
                    }
                }
            } else {
                (None, true)
            };

            let task_log = TaskLogMessage {
                task: task.clone(),
                output,
                output_complete,
            };
            tasks.insert(*task_id, task_log);
        }
    }
    Message::LogResponse(tasks)
}
