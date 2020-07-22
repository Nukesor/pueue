use ::anyhow::{Context, Result};
use ::async_std::net::TcpStream;
use ::std::env;
use ::std::io::{Read, Seek, SeekFrom, Write};
use ::std::process::Command;
use ::tempfile::NamedTempFile;

use ::pueue::message::*;
use ::pueue::protocol::*;

/// This function handles the logic for editing tasks.
/// At first, we request the daemon to send us the task to edit.
/// This also results in the task being `Locked` on the daemon side, preventing it from being
/// started or manipulated in any way, as long as we're editing.
///
/// After receiving the task information, the user can then edit it in their editor.
/// Upon exiting the text editor, the line will then be read and sent to the server
pub async fn edit(socket: &mut TcpStream, task_id: usize, edit_path: bool) -> Result<Message> {
    // Request the data to edit from the server and issue a task-lock while doing so.
    let init_message = Message::EditRequest(task_id);
    send_message(init_message, socket).await?;

    let init_response = receive_message(socket).await?;

    // In case we don't receive an EditResponse, something went wrong
    // Return the response to the parent function and let the client handle it
    // by the generic message handler.
    let init_response = if let Message::EditResponse(message) = init_response {
        message
    } else {
        return Ok(init_response);
    };

    // Edit either the path or the command, depending on the `path` flag.
    let mut command = init_response.command;
    let mut path = init_response.path;
    if edit_path {
        path = edit_line(&path)?;
    } else {
        command = edit_line(&command)?
    };

    // Create a new message with the edited command.
    let edit_message = Message::Edit(EditMessage {
        task_id,
        command,
        path,
    });
    send_message(edit_message, socket).await?;

    receive_message(socket).await
}

/// This function allows the user to edit a task's command or path.
/// Save the string to a temporary file, which is the edited by the user with $EDITOR.
/// As soon as the editor is closed, read the file content and return the line
pub fn edit_line(line: &String) -> Result<String> {
    // Create a temporary file with the command so we can edit it with the editor.
    let mut file = NamedTempFile::new().expect("Failed to create a temporary file");
    writeln!(file, "{}", line).expect("Failed writing to temporary file");

    // Start the editor on this file.
    let editor = &env::var("EDITOR").unwrap_or_else(|_e| "vi".to_string());
    Command::new(editor)
        .arg(file.path())
        .status()
        .context("Failed to start editor. Do you have the $EDITOR environment variable set?")?;

    // Read the file.
    let mut file = file.into_file();
    file.seek(SeekFrom::Start(0))
        .context("Couldn't seek to start of file. Aborting.")?;

    let mut line = String::new();
    file.read_to_string(&mut line)
        .context("Failed to read Command after editing")?;

    // Remove any trailing newlines from the command.
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }

    Ok(line)
}
