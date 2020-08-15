use ::anyhow::{anyhow, Result};
use ::std::path::{Path, PathBuf};

fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("Couldn't resolve home dir"))
}

pub fn default_config_directory() -> Result<PathBuf> {
    Ok(get_home_dir()?.join("pueue.yml"))
}

pub fn get_config_directories() -> Result<Vec<PathBuf>> {
    Ok(vec![
        // Windows Terminal stores its config file in the "AppData/Local" directory.
        dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Couldn't resolve app data directory"))?
            .join("pueue/pueue.yml"),
        default_config_directory()?,
        Path::new("./pueue.yml").to_path_buf(),
    ])
}

pub fn default_pueue_path() -> Result<String> {
    // Use local data directory since this data doesn't need to be synced.
    let path = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Couldn't resolve app data directory"))?
        .join("pueue");
    path.to_str().map_or_else(
        || Err(anyhow!("Failed to parse log path (Weird characters?)")),
        |v| Ok(v.to_string()),
    )
}
