use ::std::env;
use ::std::io::{Read, Seek, SeekFrom, Write};
use ::std::process::Command;
use ::tempfile::NamedTempFile;

use ::pueue::message::*;

use crate::cli::SubCommand;

/// This function allows the user to edit a task's command or path.
/// Save the string to a temporary file, which is the edited by the user with $EDITOR.
/// As soon as the editor is closed, read the file content and return the
/// final edit message with the updated command to the daemon.
pub fn edit(message: EditResponseMessage, cli_command: &SubCommand) -> Message {
    let edit_path = match cli_command {
        SubCommand::Edit { path, .. } => *path,
        _ => panic!(
            "Got wrong Subcommand {:?} in edit. This shouldn't happen",
            cli_command
        ),
    };

    // Edit either the path or the command, depending on the `path` flag.
    let mut command = message.command;
    let mut path = message.path;
    if edit_path {
        path = edit_line(&path);
    } else {
        command = edit_line(&command)
    };

    Message::Edit(EditMessage {
        task_id: message.task_id,
        command,
        path,
    })
}

pub fn edit_line(line: &String) -> String {
    // Create a temporary file with the command so we can edit it with the editor.
    let mut file = NamedTempFile::new().expect("Failed to create a temporary file");
    writeln!(file, "{}", line).expect("Failed writing to temporary file");

    // Start the editor on this file.
    let editor = &env::var("EDITOR").unwrap_or_else(|_e| "vi".to_string());
    Command::new(editor)
        .arg(file.path())
        .status()
        .expect("Failed to start editor. Do you have the $EDITOR environment variable set?");

    // Read the file.
    let mut file = file.into_file();
    file.seek(SeekFrom::Start(0))
        .expect("Couldn't seek to start of file. Aborting.");

    let mut line = String::new();
    file.read_to_string(&mut line)
        .expect("Failed to read Command after editing");

    // Remove any trailing newlines from the command.
    while line.ends_with('\n') || line.ends_with('\r') {
        line.pop();
    }

    line
}
