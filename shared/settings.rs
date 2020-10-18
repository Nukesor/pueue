use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;

use anyhow::{anyhow, Result};
use config::Config;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use crate::platform::directories::*;

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Shared {
    pub port: String,
    pub secret: String,
    pub pueue_directory: String,
    pub use_unix_socket: bool,
    pub unix_socket_path: String,
}

/// All settings which are used by the client
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Client {
    pub read_local_logs: bool,
    pub print_remove_warnings: bool,
}

/// All settings which are used by the daemon
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Daemon {
    pub default_parallel_tasks: usize,
    #[serde(default = "pause_on_failure_default")]
    pub pause_on_failure: bool,
    pub callback: Option<String>,
    pub groups: HashMap<String, usize>,
}

fn pause_on_failure_default() -> bool {
    false
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub shared: Shared,
    pub client: Client,
    pub daemon: Daemon,
}

impl Settings {
    /// This function creates a new configuration instance and
    /// populates it with default values for every option.
    /// If a local config file already exists it is parsed and
    /// overwrites the default option values.
    /// The local config is located at "~/.config/pueue.yml".
    pub fn new() -> Result<Settings> {
        let mut config = Config::new();

        config.set_default("shared.port", "6924")?;
        config.set_default("shared.secret", gen_random_secret())?;
        config.set_default("shared.pueue_directory", default_pueue_path()?)?;
        config.set_default("shared.use_unix_socket", false)?;
        config.set_default("shared.unix_socket_path", get_unix_socket_path()?)?;

        // Client specific config
        config.set_default("client.read_local_logs", true)?;
        config.set_default("client.print_remove_warnings", false)?;

        // Daemon specific config
        config.set_default("daemon.default_parallel_tasks", 1)?;
        config.set_default("daemon.pause_on_failure", false)?;
        config.set_default("daemon.callback", None::<String>)?;
        config.set_default("daemon.groups", HashMap::<String, i64>::new())?;

        // Add in the home config file
        parse_config(&mut config)?;

        // Try to can deserialize the entire configuration
        Ok(config.try_into()?)
    }

    /// Save the current configuration as a file to the configuration path.
    /// The file is written to the main configuration directory of the respective OS.
    pub fn save(&self) -> Result<()> {
        let config_path = default_config_directory()?.join("pueue.yml");
        let config_dir = config_path
            .parent()
            .ok_or_else(|| anyhow!("Couldn't resolve config dir"))?;

        // Create the config dir, if it doesn't exist yet
        if !config_dir.exists() {
            create_dir_all(config_dir)?;
        }

        let content = serde_yaml::to_string(self)?;
        let mut file = File::create(config_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

/// Get all possible configuration paths and check if there are
/// configuration files at those locations.
/// All configs will be merged by importance.
fn parse_config(settings: &mut Config) -> Result<()> {
    //println!("Parsing config files");
    for directory in get_config_directories()?.into_iter() {
        let path = directory.join("pueue.yml");
        //println!("Checking path: {:?}", &path);
        if path.exists() {
            //println!("Parsing config file at: {:?}", path);
            let config_file = config::File::with_name(path.to_str().unwrap());
            settings.merge(config_file)?;
        }
    }

    Ok(())
}

/// Simple helper function to generate a random secret
fn gen_random_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 20;
    let mut rng = rand::thread_rng();

    let secret: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    secret
}
