use ::std::fs::File;
use ::std::io::Read;
use ::std::time::Duration;

use ::anyhow::Result;
use ::async_std::net::TcpStream;
use ::async_std::task::sleep;

use ::pueue::log::*;
use ::pueue::message::*;
use ::pueue::protocol::send_message;
use ::pueue::state::SharedState;

/// Handle the continuous stream of a message.
pub async fn handle_follow(
    pueue_directory: &str,
    socket: &mut TcpStream,
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
                    "Multiple tasks are running, please select one of the following: {}",
                    running_ids
                )));
            }
        }
    };

    // The client requested streaming of stdout.
    let mut handle: File;
    match get_log_file_handles(task_id, pueue_directory) {
        Err(_) => {
            return Ok(create_failure_message(
                "Couldn't find output files for task. Maybe it finished? Try `log`",
            ))
        }
        Ok((stdout_handle, stderr_handle)) => {
            handle = if message.err {
                stderr_handle
            } else {
                stdout_handle
            };
        }
    }

    // Get the stdout/stderr path.
    // We need to check continuously, whether the file still exists,
    // since the file can go away (e.g. due to finishing a task).
    let (out_path, err_path) = get_log_paths(task_id, pueue_directory);
    let handle_path = if message.err { err_path } else { out_path };

    loop {
        // Check whether the file still exists. Exit if it doesn't.
        if !handle_path.exists() {
            return Ok(create_success_message(
                "File has gone away. Did somebody remove the task?",
            ));
        }
        // Read the next chunk of text from the last position.
        let mut buffer = Vec::new();

        if let Err(err) = handle.read_to_end(&mut buffer) {
            return Ok(create_failure_message(format!("Error: {}", err)));
        };
        let text = String::from_utf8_lossy(&buffer).to_string();

        // Send the new chunk and wait for 1 second.
        let response = Message::Stream(text);
        send_message(response, socket).await?;
        sleep(Duration::from_millis(1000)).await;
    }
}
