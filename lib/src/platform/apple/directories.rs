use std::path::{Path, PathBuf};

use crate::error::Error;

/// Get the default unix socket path for the current user
pub fn get_unix_socket_path() -> Result<String, Error> {
    // Create the socket in the default pueue path
    let pueue_path = PathBuf::from(default_pueue_path()?);
    let path = pueue_path.join(format!("pueue_{}.socket", whoami::username()));
    Ok(path
        .to_str()
        .ok_or(Error::InvalidPath(
            "Failed to unix socket path (Weird characters?)".into(),
        ))?
        .to_string())
}

pub fn get_home_dir() -> Result<PathBuf, Error> {
    dirs::home_dir().ok_or(Error::InvalidPath("Couldn't resolve home dir".into()))
}

pub fn default_config_directory() -> Result<PathBuf, Error> {
    Ok(get_home_dir()?.join("Library/Preferences/pueue"))
}

pub fn get_config_directories() -> Result<Vec<PathBuf>, Error> {
    Ok(vec![
        default_config_directory()?,
        Path::new(".").to_path_buf(),
    ])
}

pub fn default_pueue_path() -> Result<String, Error> {
    let path = get_home_dir()?.join(".local/share/pueue");
    Ok(path
        .to_str()
        .ok_or(Error::InvalidPath(
            "Failed to parse pueue directory path (Weird characters?)".into(),
        ))?
        .to_string())
}
