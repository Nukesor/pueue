use std::io::Write;
use std::{fs::File, path::Path};

use anyhow::{bail, Result};

/// Create a file containing the current pid of the daemon's main process.
/// Fails if it already exists or cannot be created.
pub fn create_pid_file(pueue_dir: &Path) -> Result<()> {
    let pid_file = pueue_dir.join("pueue.pid");
    if pid_file.exists() {
        bail!("Pid file already exists. \
              If you're sure there is no other daemon running please delete the file manually: {:?}", pid_file);
    }
    let mut file = File::create(pid_file)?;

    file.write_all(std::process::id().to_string().as_bytes())?;

    Ok(())
}

/// Remove the daemon's pid file.
/// Errors if it doesn't exist or cannot be deleted.
pub fn cleanup_pid_file(pueue_dir: &Path) -> Result<()> {
    let pid_file = pueue_dir.join("pueue.pid");
    if !pid_file.exists() {
        bail!(
            "Couldn't remove pid file, since it doesn't exists. This shouldn't happen: {:?}",
            pid_file
        );
    }

    std::fs::remove_file(pid_file)?;
    Ok(())
}
