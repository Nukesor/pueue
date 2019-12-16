use ::anyhow::{anyhow, Result};
use ::log::info;
use ::serde_derive::{Deserialize, Serialize};
use ::std::fs::File;
use ::std::io::prelude::*;
use ::std::path::{Path, PathBuf};
use ::rand::Rng;

use ::config::Config;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Client {
//    pub daemon_address: String,
    pub daemon_port: String,
    pub secret: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Daemon {
    pub pueue_directory: String,
    pub default_parallel_tasks: usize,
//    pub address: String,
    pub port: String,
    pub secret: String,
}


/// The struct representation of a full configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
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

        let random_secret = gen_random_secret();
        // Set pueue config defaults
        config.set_default("daemon.pueue_directory", default_pueue_path()?)?;
//        config.set_default("daemon.address", "127.0.0.1")?;
        config.set_default("daemon.port", "6924")?;
        config.set_default("daemon.default_parallel_tasks", 1)?;
        config.set_default("daemon.secret", random_secret.clone())?;

//        config.set_default("client.daemon_address", "127.0.0.1")?;
        config.set_default("client.daemon_port", "6924")?;
        config.set_default("client.secret", random_secret)?;

        // Add in the home config file
        parse_config(&mut config)?;

        // You can deserialize (and thus freeze) the entire configuration
        Ok(config.try_into()?)
    }

    /// Save the current configuration as a file to the configuration path.
    /// The file is written to "~/.config/pueue.yml".
    pub fn save(&self) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
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

fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or(anyhow!("Couldn't resolve home dir"))
}

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

#[cfg(target_os = "linux")]
fn default_config_path() -> Result<PathBuf> {
    Ok(get_home_dir()?.join(".config/pueue.yml"))
}

#[cfg(target_os = "linux")]
fn get_config_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    paths.push(Path::new("/etc/pueue.yml").to_path_buf());
    paths.push(default_config_path()?);
    paths.push(Path::new("./pueue.yml").to_path_buf());

    Ok(paths)
}

#[cfg(target_os = "linux")]
fn default_pueue_path() -> Result<String> {
    let path = get_home_dir()?.join(".local/share/pueue");
    path.to_str().map_or_else(
        || Err(anyhow!("Failed to parse log path (Weird characters?)")),
        |v| Ok(v.to_string()),
    )
}

#[cfg(target_os = "macos")]
fn default_config_path() -> Result<PathBuf> {
    Ok(get_home_dir()?.join("Library/Preferences/pueue.yml"))
}

#[cfg(target_os = "macos")]
fn get_config_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    paths.push(default_config_path()?);
    paths.push(Path::new("./pueue.yml").to_path_buf());

    Ok(paths)
}

#[cfg(target_os = "macos")]
fn default_pueue_path() -> Result<String> {
    let path = get_home_dir()?.join(".local/share/pueue");
    path.to_str().map_or_else(
        || Err(anyhow!("Failed to parse log path (Weird characters?)")),
        |v| Ok(v.to_string()),
    )
}

#[cfg(target_os = "windows")]
fn default_config_path() -> Result<PathBuf> {
    Ok(get_home_dir()?.join("AppData\\Local\\Pueue\\pueue.yml"))
}

#[cfg(target_os = "windows")]
fn get_config_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    paths.push(default_config_path()?);
    paths.push(Path::new(".\\pueue.yml").to_path_buf());

    Ok(paths)
}

#[cfg(target_os = "windows")]
fn default_pueue_path() -> Result<String> {
    let path = get_home_dir()?.join("AppData\\Local\\Pueue");
    path.to_str().map_or_else(
        || Err(anyhow!("Failed to parse log path (Weird characters?)")),
        |v| Ok(v.to_string()),
    )
}

