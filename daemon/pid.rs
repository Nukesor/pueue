use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::platform::process_helper::process_exists;

/// Read a PID file and throw an error, if another daemon instance is still running.
fn check_for_running_daemon(pid_path: &Path) -> Result<()> {
    let mut file = File::open(&pid_path).context("Failed to open PID file")?;
    let mut pid = String::new();
    file.read_to_string(&mut pid)
        .context("Failed to read PID file")?;

    let pid: u32 = pid
        .parse()
        .context(format!("Failed to parse PID from file: {pid_path:?}"))?;

    if process_exists(pid) {
        bail!(
            "Pid file already exists and another daemon seems to be running.\n\
              Please stop the daemon beforehand or delete the file manually: {pid_path:?}",
        );
    }

    Ok(())
}

/// Create a file containing the current pid of the daemon's main process.
/// Fails if it already exists or cannot be created.
pub fn create_pid_file(pueue_dir: &Path) -> Result<()> {
    let pid_path = pueue_dir.join("pueue.pid");
    // If an old PID file exists, check if the referenced process is still running.
    // The pid might not have been properly cleaned up, if the machine or Pueue crashed hard.
    if pid_path.exists() {
        check_for_running_daemon(&pid_path)?;
    }
    let mut file = File::create(pid_path)?;

    file.write_all(std::process::id().to_string().as_bytes())?;

    Ok(())
}

/// Remove the daemon's pid file.
/// Errors if it doesn't exist or cannot be deleted.
pub fn cleanup_pid_file(pueue_dir: &Path) -> Result<()> {
    let pid_file = pueue_dir.join("pueue.pid");
    if !pid_file.exists() {
        bail!(
            "Couldn't remove pid file, since it doesn't exists. This shouldn't happen: {pid_file:?}"
        );
    }

    std::fs::remove_file(pid_file)?;
    Ok(())
}
