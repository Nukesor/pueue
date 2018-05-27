use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

use toml;
use shellexpand;
use config::{ConfigError, Config, File as ConfigFile};
use users::{get_user_by_uid, get_current_uid};

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
    pub address: String,
    pub port: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Client {
    pub test: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub common: Common,
    pub client: Client,
    pub daemon: Daemon,
    pub worker: Worker,
}

const config_path: &str = "~/.config/pueue.toml";

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        // Get user and group id information
        let user = get_user_by_uid(get_current_uid()).unwrap();
        let group_id = user.primary_group_id() as i64;

        // Set pueue config defaults
        config.set_default("common.local_socket_dir", "/tmp/")?;
        config.set_default("common.group_id", group_id)?;

        config.set_default("worker.local", true)?;
        config.set_default("worker.address", "127.0.0.1")?;
        config.set_default("worker.port", 2112)?;

        config.set_default("daemon.parallel_worker", 1)?;

        config.set_default("client.test", "he")?;

        // Add in the home config file
        let path = shellexpand::tilde(config_path).into_owned();
        config.merge(ConfigFile::with_name(&path).required(false))?;

        // You can deserialize (and thus freeze) the entire configuration
        config.try_into()
    }

    pub fn save(&self) -> Result<(), Box<Error>> {
        let content = toml::to_string(self).unwrap();

        let path = shellexpand::tilde(config_path).into_owned();
        let mut file = File::create(&path)?;

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
