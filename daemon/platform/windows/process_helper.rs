use anyhow::{bail, Result};
use std::process::Child;

use crate::task_handler::ProcessAction;
use log::info;

/// Send a signal to a windows process.
pub fn send_signal_to_child(
    _child: &Child,
    _action: &ProcessAction,
    _children: bool,
) -> Result<bool> {
    bail!("not supported on windows.")
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
