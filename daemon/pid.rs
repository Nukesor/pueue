use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};
use log::info;
use pueue_lib::error::Error;

use crate::platform::process_helper::process_exists;

/// Read a PID file and throw an error, if another daemon instance is still running.
fn check_for_running_daemon(pid_path: &Path) -> Result<()> {
    info!("Placing pid file at {pid_path:?}");
    let mut file =
        File::open(&pid_path).map_err(|err| Error::IoError("opening pid file".to_string(), err))?;
    let mut pid = String::new();
    file.read_to_string(&mut pid)
        .map_err(|err| Error::IoError("reading pid file".to_string(), err))?;

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
pub fn create_pid_file(pid_path: &Path) -> Result<()> {
    // If an old PID file exists, check if the referenced process is still running.
    // The pid might not have been properly cleaned up, if the machine or Pueue crashed hard.
    if pid_path.exists() {
        check_for_running_daemon(pid_path)?;
    }
    let mut file = File::create(pid_path)
        .map_err(|err| Error::IoError("creating pid file".to_string(), err))?;

    file.write_all(std::process::id().to_string().as_bytes())
        .map_err(|err| Error::IoError("writing pid file".to_string(), err))?;

    Ok(())
}

/// Remove the daemon's pid file.
/// Errors if it doesn't exist or cannot be deleted.
pub fn cleanup_pid_file(pid_path: &Path) -> Result<()> {
    std::fs::remove_file(pid_path)
        .map_err(|err| Error::IoError("removing pid file".to_string(), err))?;
    Ok(())
}
