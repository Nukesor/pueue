use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("Couldn't resolve home dir"))
}

pub fn default_config_directory() -> Result<PathBuf> {
    Ok(dirs::data_local_dir()
        .ok_or(anyhow!("Couldn't resolve app data directory"))?
        .join("pueue"))
}

pub fn get_config_directories() -> Result<Vec<PathBuf>> {
    Ok(vec![
        // Windows Terminal stores its config file in the "AppData/Local" directory.
        default_config_directory()?,
        Path::new(".").to_path_buf(),
    ])
}

pub fn default_pueue_path() -> Result<String> {
    // Use local data directory since this data doesn't need to be synced.
    let path = dirs::data_local_dir()
        .ok_or(anyhow!("Couldn't resolve app data directory"))?
        .join("pueue");
    Ok(path
        .to_str()
        .ok_or(anyhow!("Failed to parse log path (Weird characters?)"))?
        .to_string())
}
