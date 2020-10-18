use anyhow::{bail, Result};
use std::process::{Child, Command};

use crate::task_handler::ProcessAction;
use log::info;

pub fn compile_shell_command(command_string: &str) -> Command {
    // Chain two `powershell` commands, one that sets the output encoding to utf8 and then the user provided one.
    let mut command = Command::new("powershell");
    command.arg("-c").arg(format!(
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::UTF8; {}",
        command_string
    ));

    command
}

/// Send a signal to a windows process.
pub fn send_signal_to_child(
    _child: &Child,
    action: &ProcessAction,
    _children: bool,
) -> Result<bool> {
    match action {
        ProcessAction::Pause => bail!("Pause is not yet supported on windows."),
        ProcessAction::Resume => bail!("Resume is not yet supported on windows."),
        ProcessAction::Kill => bail!("Kill is not yet supported on windows."),
    }
}

/// Kill a child process
pub fn kill_child(task_id: usize, child: &mut Child, _kill_children: bool) -> bool {
    match child.kill() {
        Err(_) => {
            info!("Task {} has already finished by itself", task_id);
            false
        }
        Ok(_) => true,
    }
}
