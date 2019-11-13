use ::std::fs::File;
use ::std::io::prelude::*;
use ::std::collections::HashMap;
use ::std::path::{Path, PathBuf};
use ::anyhow::{Result, anyhow};
use ::log::info;
use ::serde_derive::{Deserialize, Serialize};

use ::config::Config;
use ::toml;

#[derive(Debug, Deserialize, Serialize)]
pub struct Client {
    pub daemon_address: String,
    pub daemon_port: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Daemon {
    pub default_worker_count: u32,
    pub address: String,
    pub port: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub name: String,
    pub worker_count: u32,
}

/// The struct representation of a full configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub client: Client,
    pub daemon: Daemon,
    pub groups: HashMap<String, Group>,
}

impl Settings {
    /// This function creates a new configuration instance and
    /// populates it with default values for every option.
    /// If a local config file already exists it is parsed and
    /// overwrites the default option values.
    /// The local config is located at "~/.config/pueue.yml".
    pub fn new() -> Result<Settings> {
        let mut config = Config::new();

        // Set pueue config defaults
        config.set_default("daemon.address", "127.0.0.1")?;
        config.set_default("daemon.port", "6924")?;
        config.set_default("daemon.default_parallel_worker", 1)?;

        config.set_default("client.daemon_address", "127.0.0.1")?;
        config.set_default("client.daemon_port", "6924")?;

        // Add in the home config file
        parse_config(&mut config)?;

        // You can deserialize (and thus freeze) the entire configuration
        Ok(config.try_into()?)
    }

    /// Save the current configuration as a file to the configuration path.
    /// The file is written to "~/.config/pueue.yml".
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string(self).unwrap();

        let mut file = File::create(default_config_path()?)?;

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

fn parse_config(settings: &mut Config) -> Result<()> {
    info!("Parsing config files");
    let config_paths = get_config_paths()?;

    for path in config_paths.into_iter() {
        info!("Checking path: {:?}", &path);
        if path.exists() {
            info!("Parsing config file at: {:?}", path);
            let config_file = config::File::with_name(path.to_str().unwrap());
            settings.merge(config_file)?;
        }
    }

    Ok(())
}



#[cfg(target_os = "linux")]
fn default_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(anyhow!("Couldn't resolve home dir"))?;
    Ok(home_dir.join(".config/pueue.yml"))
}

#[cfg(target_os = "linux")]
fn get_config_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let home_dir = dirs::home_dir().ok_or(anyhow!("Couldn't resolve home dir"))?;
    paths.push(Path::new("/etc/pueue.yml").to_path_buf());
    paths.push(home_dir.join(".config/pueue.yml"));
    paths.push(Path::new("./pueue.yml").to_path_buf());

    Ok(paths)
}
