use std::path::PathBuf;

use crate::settings::expand_home;

fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

pub fn default_pueue_path() -> PathBuf {
    if let Ok(path) = std::env::var("XDG_DATA_HOME") {
        expand_home(&PathBuf::from(path)).join("pueue")
    } else {
        get_home_dir().join(".local/share/pueue")
    }
}

/// Try to find `XDG_RUNTIME_DIR` in the environment.
pub fn default_runtime_directory() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("XDG_RUNTIME_DIR") {
        Some(expand_home(&PathBuf::from(path)))
    } else {
        None
    }
}

pub fn get_config_directories() -> Vec<PathBuf> {
    vec![default_config_directory(), PathBuf::from(".")]
}

/// Return the default config directory for pueue.
/// This follows the XDG specification and uses `XDG_CONFIG_HOME` if it's set.
pub fn default_config_directory() -> PathBuf {
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(path).join("pueue")
    } else {
        get_home_dir().join(".config/pueue")
    }
}
