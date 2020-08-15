use anyhow::{bail, Result};

/// Send a signal to a windows process.
pub fn send_signal(pid: u32, action: ProcessAction, children: bool) -> Result<bool> {
    bail!("not supported on windows.")
}

/// Get all children of a specific process
pub fn get_children(pid: i32) -> Option<Vec<i32>> {
    None
}

/// A small helper that sends a signal to all children of a specific process by id.
pub fn send_signal_to_children(pid: i32, action: ProcessAction) {
    error!(
        "Calling send_signal_to_children on task {} on windows. This shouldn't happen!",
        task_id
    )
}
