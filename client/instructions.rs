use ::log::error;
use ::std::env;
use ::std::io::{Read, Seek, SeekFrom, Write};
use ::std::process::exit;
use ::std::process::Command;
use ::tempfile::NamedTempFile;

use ::pueue::message::*;

/// This function allows the user to edit a task's command or path
/// Save the string to a temporary file, which is the edited by the user with $EDITOR
/// As soon as the editor is closed, read the file content and return the
/// final edit message with the updated command to the daemon
pub fn edit(message: Message, edit_path: bool) -> Message {
    let message = match message {
        Message::EditResponse(message) => message,
        _ => {
            error!("Should have received a EditResponseMessage");
            exit(1);
        }
    };

    let mut command = message.command;
    let mut path = message.path;
    let mut to_edit = if edit_path {
        path.clone()
    } else {
        command.clone()
    };

    // Create a temporary file with the command, vim can edit
    let mut file = NamedTempFile::new().expect("Failed to create a temporary file");
    writeln!(file, "{}", to_edit).expect("Failed writing to temporary file");

    // Start the editor on this file
    let editor = env::var("EDITOR").unwrap_or("vi".to_string());
    Command::new(editor)
        .arg(file.path())
        .status()
        .expect("Failed to start editor");

    // Read the file
    let mut file = file.into_file();
    file.seek(SeekFrom::Start(0))
        .expect("Couldn't seek to start of file. Aborting.");
    to_edit = String::new();
    file.read_to_string(&mut to_edit)
        .expect("Failed to read Command after editing");

    // Remove any trailing newlines from the command
    while to_edit.ends_with('\n') || to_edit.ends_with('\r') {
        to_edit.pop();
    }

    if edit_path {
        path = to_edit
    } else {
        command = to_edit
    }

    Message::Edit(EditMessage {
        task_id: message.task_id,
        command: command,
        path: path,
    })
}
