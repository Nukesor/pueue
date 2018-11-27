use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

use config::{Config, ConfigError, File as ConfigFile};
use shellexpand;
use toml;
use users::{get_current_uid, get_user_by_uid};

#[derive(Debug, Deserialize, Serialize)]
pub struct Common {
    pub local_socket_dir: String,
    pub group_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Daemon {
    pub parallel_worker: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Worker {
    pub local: bool,
    pub worker_group: String,
    pub address: String,
    pub port: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Client {
    pub test: String,
}

/// The struct representation of a full configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub common: Common,
    pub client: Client,
    pub daemon: Daemon,
    pub worker: Worker,
}

const CONFIG_PATH: &str = "~/.config/pueue.toml";

impl Settings {
    /// This function creates a new configuration instance and
    /// populates it with default values for every option.
    /// If a local config file already exists it is parsed and
    /// overwrites the default option values.
    /// The local config is located at "~/.config/pueue.toml".
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        // Get user and group id information
        let user = get_user_by_uid(get_current_uid()).unwrap();
        let group_id = user.primary_group_id() as i64;

        // Set pueue config defaults
        config.set_default("common.local_socket_dir", "/tmp/")?;
        config.set_default("common.group_id", group_id)?;

        config.set_default("worker.local", true)?;
        config.set_default("worker.worker_group", "local")?;
        config.set_default("worker.address", "127.0.0.1")?;
        config.set_default("worker.port", 2112)?;

        config.set_default("daemon.parallel_worker", 1)?;

        config.set_default("client.test", "he")?;

        // Add in the home config file
        let path = shellexpand::tilde(CONFIG_PATH).into_owned();
        config.merge(ConfigFile::with_name(&path).required(false))?;

        // You can deserialize (and thus freeze) the entire configuration
        config.try_into()
    }

    /// Save the current configuration as a file to the configuration path.
    /// The file is written to "~/.config/pueue.toml".
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let content = toml::to_string(self).unwrap();

        let path = shellexpand::tilde(CONFIG_PATH).into_owned();
        let mut file = File::create(&path)?;

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
