use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// Get the default unix socket path for the current user
pub fn get_unix_socket_path() -> Result<String> {
    // Create the socket in the default pueue path
    let pueue_path = PathBuf::from(default_pueue_path()?);
    let path = pueue_path.join(format!("pueue_{}.socket", whoami::username()));
    Ok(path
        .to_str()
        .ok_or_else(|| anyhow!("Failed to parse log path (Weird characters?)"))?
        .to_string())
}

pub fn get_home_dir() -> Result<PathBuf> {
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
        .ok_or_else(|| anyhow!("Failed to parse log path (Weird characters?)"))?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{create_dir_all, remove_file, File};
    use std::io::prelude::*;

    use anyhow::Result;

    #[test]
    fn test_create_unix_socket() -> Result<()> {
        let path = get_unix_socket_path()?;
        create_dir_all(default_pueue_path()?)?;

        // If pueue is currently running on the system, simply accept that we found the correct path
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
