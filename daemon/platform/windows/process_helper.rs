use anyhow::{bail, Result};

use crate::task_handler::ProcessAction;

/// Send a signal to a windows process.
pub fn send_signal(_pid: u32, _action: &ProcessAction, _children: bool) -> Result<bool> {
    bail!("not supported on windows.")
}

/// Get all children of a specific process.
/// The Vec<i32> is just a placeholder.
pub fn get_children(_pid: i32) -> Option<Vec<i32>> {
    None
}

/// Send a signal to multiple processes.
/// The Vec<i32> is just a placeholder.
pub fn send_signal_to_processes(_processes: Vec<i32>, _action: &ProcessAction) {}
