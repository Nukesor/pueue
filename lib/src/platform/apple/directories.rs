use std::path::PathBuf;

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

pub fn default_pueue_path() -> PathBuf {
    get_home_dir().join(".local/share/pueue")
}

pub fn default_runtime_directory() -> Option<PathBuf> {
    None
}

pub fn get_config_directories() -> Vec<PathBuf> {
    vec![default_config_directory(), PathBuf::from(".")]
}

pub fn default_config_directory() -> PathBuf {
    get_home_dir().join("Library/Preferences/pueue")
}
