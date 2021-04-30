use std::path::{Path, PathBuf};

use crate::error::Error;

/// Get the default unix socket path for the current user
pub fn get_unix_socket_path() -> Result<String, Error> {
    // Create the socket in the default pueue path
    let pueue_path = PathBuf::from(default_pueue_path()?);
    let path = pueue_path.join(format!("pueue_{}.socket", whoami::username()));
    Ok(path
        .to_str()
        .ok_or_else(|| {
            Error::InvalidPath("Failed to parse unix socket (Weird characters?)".into())
        })?
        .to_string())
}

fn get_home_dir() -> Result<PathBuf, Error> {
    dirs::home_dir().ok_or_else(|| Error::InvalidPath("Couldn't resolve home dir".into()))
}

pub fn default_config_directory() -> Result<PathBuf, Error> {
    Ok(get_home_dir()?.join(".config/pueue"))
}

pub fn get_config_directories() -> Result<Vec<PathBuf>, Error> {
    Ok(vec![
        Path::new("/etc/pueue").to_path_buf(),
        default_config_directory()?,
        Path::new(".").to_path_buf(),
    ])
}

pub fn default_pueue_path() -> Result<String, Error> {
    let path = get_home_dir()?.join(".local/share/pueue");
    Ok(path
        .to_str()
        .ok_or_else(|| {
            Error::InvalidPath("Failed to parse pueue directory path (Weird characters?)".into())
        })?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{create_dir_all, remove_file, File};
    use std::io::prelude::*;

    #[test]
    fn test_create_unix_socket() -> Result<(), Error> {
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
