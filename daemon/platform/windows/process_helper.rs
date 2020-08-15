use anyhow::{bail, Result};

use crate::task_handler::ProcessAction;

/// Send a signal to a windows process.
pub fn send_signal(pid: u32, action: &ProcessAction, children: bool) -> Result<bool> {
    bail!("not supported on windows.")
}

/// Get all children of a specific process
pub fn get_children(pid: i32) -> Option<Vec<i32>> {
    None
}

/// Send a signal to multiple processes
pub fn send_signal_to_processes(pid: i32, action: &ProcessAction) {}
