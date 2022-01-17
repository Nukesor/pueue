use std::io::Read;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;

use pueue_lib::log::*;
use pueue_lib::network::message::*;
use pueue_lib::network::protocol::{send_message, GenericStream};
use pueue_lib::state::SharedState;

/// Handle the continuous stream of a message.
pub async fn handle_follow(
    pueue_directory: &Path,
    stream: &mut GenericStream,
    state: &SharedState,
    message: StreamRequestMessage,
) -> Result<Message> {
    // The user can specify the id of the task they want to follow
    // If the id isn't specified and there's only a single running task, this task will be used.
    // However, if there are multiple running tasks, the user will have to specify an id.
    let task_id = if let Some(task_id) = message.task_id {
        task_id
    } else {
        // Get all ids of running tasks
        let state = state.lock().unwrap();
        let running_ids: Vec<_> = state
            .tasks
            .iter()
            .filter_map(|(&id, t)| if t.is_running() { Some(id) } else { None })
            .collect();

        // Return a message on "no" or multiple running tasks.
        match running_ids.len() {
            0 => {
                return Ok(create_failure_message("There are no running tasks."));
            }
            1 => running_ids[0],
            _ => {
                let running_ids = running_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Ok(create_failure_message(format!(
                    "Multiple tasks are running, please select one of the following: {running_ids}"
                )));
            }
        }
    };

    let mut handle = match get_log_file_handle(task_id, pueue_directory) {
        Err(_) => {
            return Ok(create_failure_message(
                "Couldn't find output files for task. Maybe it finished? Try `log`",
            ))
        }
        Ok(handle) => handle,
    };

    // Get the output path.
    // We need to check continuously, whether the file still exists,
    // since the file can go away (e.g. due to finishing a task).
    let path = get_log_path(task_id, pueue_directory);

    // If lines is passed as an option, seek the output file handle to the start of
    // the line corresponding to the `lines` number of lines from the end of the file.
    // The loop following this section will copy those lines to stdout.
    if let Some(lines) = message.lines {
        if let Err(err) = seek_to_last_lines(&mut handle, lines) {
            println!("Error seeking to last lines from log: {err}");
        }
    }
    loop {
        // Check whether the file still exists. Exit if it doesn't.
        if !path.exists() {
            return Ok(create_success_message(
                "File has gone away. Did somebody remove the task?",
            ));
        }
        // Read the next chunk of text from the last position.
        let mut buffer = Vec::new();

        if let Err(err) = handle.read_to_end(&mut buffer) {
            return Ok(create_failure_message(format!("Error: {err}")));
        };
        let text = String::from_utf8_lossy(&buffer).to_string();

        // Send the new chunk and wait for 1 second.
        let response = Message::Stream(text);
        send_message(response, stream).await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
