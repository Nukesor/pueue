use std::env;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use tempfile::NamedTempFile;

use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;
use pueue_lib::process_helper::compile_shell_command;

/// This function handles the logic for editing tasks.
/// At first, we request the daemon to send us the task to edit.
/// This also results in the task being `Locked` on the daemon side, preventing it from being
/// started or manipulated in any way, as long as we're editing.
///
/// After receiving the task information, the user can then edit it in their editor.
/// Upon exiting the text editor, the line will then be read and sent to the server
pub async fn edit(
    stream: &mut GenericStream,
    task_id: usize,
    edit_command: bool,
    edit_path: bool,
    edit_label: bool,
) -> Result<Message> {
    // Request the data to edit from the server and issue a task-lock while doing so.
    let init_message = Message::EditRequest(task_id);
    send_message(init_message, stream).await?;

    let init_response = receive_message(stream).await?;

    // In case we don't receive an EditResponse, something went wrong
    // Return the response to the parent function and let the client handle it
    // by the generic message handler.
    let Message::EditResponse(init_response ) = init_response else {
        return Ok(init_response);
    };

    // Edit the command if explicitly specified or if no flags are provided (the default)
    let edit_command = edit_command || !edit_path && !edit_label;

    // Edit all requested properties.
    let edit_result = edit_task_properties(
        &init_response.command,
        &init_response.path,
        &init_response.label,
        edit_command,
        edit_path,
        edit_label,
    );

    // Any error while editing will result in the client aborting the editing process.
    // However, as the daemon moves tasks that're edited into the `Locked` state, we cannot simply
    // exit the client. We rather have to notify the daemon that the editing process was interrupted.
    // In the following, we notify the daemon of any errors, so it can restore the task to its previous state.
    let edited_props = match edit_result {
        Ok(inner) => inner,
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

            return Err(error);
        }
    };

    // Create a new message with the edited properties.
    let edit_message = EditMessage {
        task_id,
        command: edited_props.command,
        path: edited_props.path,
        label: edited_props.label,
        delete_label: edited_props.delete_label,
    };
    send_message(edit_message, stream).await?;

    Ok(receive_message(stream).await?)
}

#[derive(Default)]
pub struct EditedProperties {
    pub command: Option<String>,
    pub path: Option<PathBuf>,
    pub label: Option<String>,
    pub delete_label: bool,
}

/// Takes several task properties and edit them if requested.
/// The `edit_*` booleans are used to determine which fields should be edited.
///
/// Fields that have been edited will be returned as their `Some(T)` equivalent.
///
/// The returned values are: `(command, path, label)`
pub fn edit_task_properties(
    original_command: &str,
    original_path: &Path,
    original_label: &Option<String>,
    edit_command: bool,
    edit_path: bool,
    edit_label: bool,
) -> Result<EditedProperties> {
    let mut props = EditedProperties::default();

    // Update the command if requested.
    if edit_command {
        props.command = Some(edit_line(original_command)?);
    };

    // Update the path if requested.
    if edit_path {
        let str_path = original_path
            .to_str()
            .context("Failed to convert task path to string")?;
        let changed_path = edit_line(str_path)?;
        props.path = Some(PathBuf::from(changed_path));
    }

    // Update the label if requested.
    if edit_label {
        let edited_label = edit_line(&original_label.clone().unwrap_or_default())?;

        // If the user deletes the label in their editor, an empty string will be returned.
        // This is an indicator that the task should no longer have a label, in which case we
        // set the `delete_label` flag.
        if edited_label.is_empty() {
            props.delete_label = true;
        } else {
            props.label = Some(edited_label);
        };
    }

    Ok(props)
}

/// This function enables the user to edit a task's details.
/// Save any string to a temporary file, which is opened in the specified `$EDITOR`.
/// As soon as the editor is closed, read the file content and return the line.
fn edit_line(line: &str) -> Result<String> {
    // Create a temporary file with the command so we can edit it with the editor.
    let mut file = NamedTempFile::new().expect("Failed to create a temporary file");
    writeln!(file, "{line}").context("Failed to write to temporary file.")?;

    // Get the editor that should be used from the environment.
    let editor = match env::var("EDITOR") {
        Err(_) => bail!("The '$EDITOR' environment variable couldn't be read. Aborting."),
        Ok(editor) => editor,
    };

    // Compile the command to start the editor on the temporary file.
    // We escape the file path for good measure, but it shouldn't be necessary.
    let path = shell_escape::escape(file.path().to_string_lossy());
    let editor_command = format!("{editor} {path}");
    let status = compile_shell_command(&editor_command)
        .status()
        .context("Editor command did somehow fail. Aborting.")?;

    if !status.success() {
        bail!("The editor exited with a non-zero code. Aborting.");
    }

    // Read the file.
    let mut file = file.into_file();
    file.rewind()
        .context("Couldn't seek to start of file. Aborting.")?;

    let mut line = String::new();
    file.read_to_string(&mut line)
        .context("Failed to read Command after editing")?;

    // Remove any trailing newlines from the command.
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }

    Ok(line.trim().to_string())
}
