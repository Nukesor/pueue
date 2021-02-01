use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// Get the default unix socket path for the current user
pub fn get_unix_socket_path() -> Result<String> {
    // Create the socket in the default pueue path
    let pueue_path = PathBuf::from(default_pueue_path()?);
    let path = pueue_path.join(format!("pueue_{}.socket", whoami::username()));
    Ok(path
        .to_str()
        .ok_or(anyhow!("Failed to parse log path (Weird characters?)"))?
        .to_string())
}

fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("Couldn't resolve home dir"))
}

pub fn default_config_directory() -> Result<PathBuf> {
    Ok(get_home_dir()?.join("Library/Preferences/pueue"))
}

pub fn get_config_directories() -> Result<Vec<PathBuf>> {
    Ok(vec![
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
