use std::env;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use tempfile::NamedTempFile;

use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;

/// This function handles the logic for editing tasks.
/// At first, we request the daemon to send us the task to edit.
/// This also results in the task being `Locked` on the daemon side, preventing it from being
/// started or manipulated in any way, as long as we're editing.
///
/// After receiving the task information, the user can then edit it in their editor.
/// Upon exiting the text editor, the line will then be read and sent to the server
pub async fn edit(stream: &mut GenericStream, task_id: usize, edit_path: bool) -> Result<Message> {
    // Request the data to edit from the server and issue a task-lock while doing so.
    let init_message = Message::EditRequest(task_id);
    send_message(init_message, stream).await?;

    let init_response = receive_message(stream).await?;

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
        let str_path = path
            .to_str()
            .context("Failed to convert task path to string")?;
        let changed_path = edit_line_wrapper(stream, task_id, str_path).await?;
        path = PathBuf::from(changed_path);
    } else {
        command = edit_line_wrapper(stream, task_id, &command).await?
    };

    // Create a new message with the edited command.
    let edit_message = Message::Edit(EditMessage {
        task_id,
        command,
        path,
    });
    send_message(edit_message, stream).await?;

    Ok(receive_message(stream).await?)
}

/// This function wraps the edit_line function for error handling.
///
/// Any error will result in the client aborting the editing process.
/// This includes notifying the daemon of this, so it can restore the task to its previous state.
pub async fn edit_line_wrapper(
    stream: &mut GenericStream,
    task_id: usize,
    line: &str,
) -> Result<String> {
    match edit_line(line) {
        Ok(edited_line) => Ok(edited_line),
        Err(error) => {
            eprintln!("Encountered an error while editing. Trying to restore the task's status.");
            // Notify the daemon that something went wrong.
            let edit_message = Message::EditRestore(task_id);
            send_message(edit_message, stream).await?;
            let response = receive_message(stream).await?;
            match response {
                Message::Failure(message) | Message::Success(message) => {
                    eprintln!("{message}");
                }
                _ => eprintln!("Received unknown resonse: {response:?}"),
            };

            Err(error)
        }
    }
}

/// This function allows the user to edit a task's command or path.
/// Save the string to a temporary file, which is the edited by the user with $EDITOR.
/// As soon as the editor is closed, read the file content and return the line
fn edit_line(line: &str) -> Result<String> {
    // Create a temporary file with the command so we can edit it with the editor.
    let mut file = NamedTempFile::new().expect("Failed to create a temporary file");
    writeln!(file, "{}", line).context("Failed to write to temporary file.")?;

    // Start the editor on this file.
    let editor = match env::var("EDITOR") {
        Err(_) => bail!("The '$EDITOR' environment variable couldn't be read. Aborting."),
        Ok(editor) => editor,
    };

    let status = Command::new(editor)
        .arg(file.path())
        .status()
        .context("Editor command did somehow fail. Aborting.")?;

    if !status.success() {
        bail!("The editor exited with a non-zero code. Aborting.");
    }

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
