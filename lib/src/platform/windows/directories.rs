use std::path::{Path, PathBuf};

use crate::error::Error;

pub fn get_home_dir() -> Result<PathBuf, Error> {
    dirs::home_dir().ok_or(Error::InvalidPath("Couldn't resolve home dir".into()))
}

pub fn default_config_directory() -> Result<PathBuf, Error> {
    Ok(dirs::data_local_dir()
        .ok_or(Error::InvalidPath(
            "Couldn't resolve app data directory".into(),
        ))?
        .join("pueue"))
}

pub fn get_config_directories() -> Result<Vec<PathBuf>, Error> {
    Ok(vec![
        // Windows Terminal stores its config file in the "AppData/Local" directory.
        default_config_directory()?,
        Path::new(".").to_path_buf(),
    ])
}

pub fn default_pueue_path() -> Result<String, Error> {
    // Use local data directory since this data doesn't need to be synced.
    let path = dirs::data_local_dir()
        .ok_or(Error::InvalidPath(
            "Couldn't resolve app data directory".into(),
        ))?
        .join("pueue");
    Ok(path
        .to_str()
        .ok_or(Error::InvalidPath(
            "Failed to parse pueue directory path (Weird characters?)".into(),
        ))?
        .to_string())
}
