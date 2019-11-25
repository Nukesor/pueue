use ::std::env;
use ::std::process::Command;
use ::std::io::{Write, Read, Seek, SeekFrom};
use ::std::process::exit;
use ::tempfile::NamedTempFile;
use ::log::error;

use ::pueue::message::*;


/// Save the command tokjkjkjkjkj
pub fn edit(message: Message) -> Message {
    let message = match message {
        Message::EditResponse(message) => message,
        _ => {
            error!("Should have received a EditResponseMessage");
            exit(1);
        }
                ,
    };

    let editor = env::var("EDITOR").unwrap_or("vi".to_string());

    // Create a temporary file with the command, vim can edit
    let mut file = NamedTempFile::new().expect("Failed to create a temporary file");
    writeln!(file, "{}", message.command).expect("Failed writing to temporary file");
    Command::new(editor).arg(file.path()).status().expect("Failed to start editor") ;

    let mut file = file.into_file();
    file.seek(SeekFrom::Start(0)).expect("Couldn't seek to start of file. Aborting.");
    let mut edited_command = String::new();
    file.read_to_string(&mut edited_command).expect("Failed to read Command after editing");

    Message::Edit(EditMessage {
        task_id: message.task_id,
        command: edited_command,
    })
}
