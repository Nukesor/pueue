use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use users::{get_current_uid, get_user_by_uid};

/// Get the default unix socket path for the current user
pub fn get_unix_socket_path() -> Result<String> {
    // Get the user and their username
    let user = get_user_by_uid(get_current_uid())
        .ok_or(anyhow!("Couldn't find username for current user"))?;
    let username = user.name().to_string_lossy();

    // Create the socket in the default /tmp/ directory
    let path = Path::new("/tmp/").join(format!("pueue_{}.socket", username));
    Ok(path.to_string_lossy().into())
}

fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("Couldn't resolve home dir"))
}

pub fn default_config_directory() -> Result<PathBuf> {
    Ok(get_home_dir()?.join(".config/pueue"))
}

pub fn get_config_directories() -> Result<Vec<PathBuf>> {
    Ok(vec![
        Path::new("/etc/pueue").to_path_buf(),
        default_config_directory()?,
        Path::new(".").to_path_buf(),
    ])
}

pub fn default_pueue_path() -> Result<String> {
    let path = get_home_dir()?.join(".local/share/pueue");
    Ok(path
        .to_str()
        .ok_or(anyhow!("Failed to parse log path (Weird characters?)"))?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{remove_file, File};
    use std::io::prelude::*;

    use anyhow::Result;

    #[test]
    fn test_create_unix_socket() -> Result<()> {
        let path = get_unix_socket_path()?;

        // If pueue is currently running on the system,
        // simply accept that we found the correct path
        if PathBuf::from(&path).exists() {
            return Ok(());
        }

        // Otherwise try to create it and write to it
        let mut file = File::create(&path)?;
        assert!(file.write_all(b"Hello, world!").is_ok());

        remove_file(&path)?;

        Ok(())
    }
}
