use ::anyhow::Result;
use ::async_std::future;
use ::async_std::net::TcpStream;
use ::async_std::prelude::*;
use ::std::fs::File;
use ::std::io::Read;
use ::std::time::Duration;

use ::pueue::log::*;
use ::pueue::message::*;
use ::pueue::protocol::send_message;
use ::pueue::settings::Settings;

/// Handle the continuous stream of a message
pub async fn handle_show(
    settings: &Settings,
    socket: &mut TcpStream,
    message: StreamRequestMessage,
) -> Result<Message> {
    if message.follow || message.err {
        // The client requested streaming of stdout
        let mut handle: File;
        match get_log_file_handles(message.task_id, &settings) {
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

        // Get the stdout/stderr path
        // We need to check continuously, whether the file still exists,
        // since the file can go away (e.g. due to finishing a task)
        let (out_path, err_path) = get_log_paths(message.task_id, settings);
        let handle_path = if message.err { err_path } else { out_path };

        loop {
            // Check whether the file still exists. Exit if it doesn't
            if !handle_path.exists() {
                return Ok(create_success_message(
                    "File has gone away. The task probably just finished",
                ));
            }
            // Read the next chunk of text from the last position
            let mut text = String::new();
            if let Err(err) = handle.read_to_string(&mut text) {
                return Ok(create_failure_message(format!("Error: {}", err)));
            };

            // Send the new chunk and wait for 1 second
            let response = Message::Stream(text);
            send_message(&response, socket).await?;
            let wait = future::ready(1).delay(Duration::from_millis(1000));
            wait.await;
        }
    } else {
        // The client requested a one-shot execution
        // Simply read the file and send the current stdout/stderr once
        let stdout: String;
        let stderr: String;
        match read_log_files(message.task_id, &settings) {
            Err(_) => {
                return Ok(create_failure_message(
                    "Couldn't find output files for task. Maybe it finished? Try `log`",
                ))
            }
            Ok((stdout_text, stderr_text)) => {
                stdout = stdout_text;
                stderr = stderr_text;
            }
        }

        let response = format!("Stdout:\n{}\n\nStderr:\n{}", stdout, stderr);

        Ok(create_success_message(response))
    }
}
