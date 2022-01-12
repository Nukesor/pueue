use std::path::PathBuf;

// Use local data directory since this data doesn't need to be synced.
pub fn data_local_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("\\"))
}

pub fn default_pueue_path() -> PathBuf {
    data_local_dir().join("pueue")
}

pub fn default_runtime_directory() -> Option<PathBuf> {
    None
}

pub fn default_config_directory() -> PathBuf {
    data_local_dir().join("pueue")
}

pub fn get_config_directories() -> Vec<PathBuf> {
    vec![
        // Windows Terminal stores its config file in the "AppData/Local" directory.
        default_config_directory(),
        PathBuf::from("."),
    ]
}
