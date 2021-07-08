use std::collections::{BTreeMap, HashMap};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use config::Config;
use log::info;
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

use crate::error::Error;
use crate::platform::directories::*;

/// All settings which are used by both, the client and the daemon
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Shared {
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The directory that is used for all runtime information. \
    /// I.e. task logs, sockets, state dumps, etc.
    pub pueue_directory: PathBuf,
    /// If this is set to true, unix sockets will be used.
    /// Otherwise we default to TCP+TLS
    #[cfg(not(target_os = "windows"))]
    pub use_unix_socket: bool,
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The path to the unix socket.
    #[cfg(not(target_os = "windows"))]
    pub unix_socket_path: PathBuf,

    /// The TCP hostname/ip address.
    pub host: String,
    /// The TCP port.
    pub port: String,
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The path to the TLS certificate used by the daemon. \
    /// This is also used by the client to verify the daemon's identity.
    pub daemon_cert: PathBuf,
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The path to the TLS key used by the daemon.
    pub daemon_key: PathBuf,
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The path to the file containing the shared secret used to authenticate the client.
    pub shared_secret_path: PathBuf,
}

/// All settings which are used by the client
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Client {
    /// If set to true, all tasks will be restart in place, instead of creating a new task.
    /// False is the default, as you'll lose the logs of the previously failed tasks when
    /// restarting tasks in place.
    pub restart_in_place: bool,
    /// Whether the client should read the logs directly from disk or whether it should
    /// request the data from the daemon via socket.
    pub read_local_logs: bool,
    /// Whether the client should show a confirmation question on potential dangerous actions.
    pub show_confirmation_questions: bool,
    /// Whether aliases specified in `pueue_aliases.yml` should be expanded in the `pueue status`
    /// or shown in their short form.
    pub show_expanded_aliases: bool,
    /// Whether the client should use dark shades instead of regular colors.
    pub dark_mode: bool,
    /// The max amount of lines each task get's in the `pueue status` view.
    pub max_status_lines: Option<usize>,
    /// The format that will be used to display time formats in `pueue status`.
    pub status_time_format: String,
    /// The format that will be used to display datetime formats in `pueue status`.
    pub status_datetime_format: String,
}

/// All settings which are used by the daemon
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Daemon {
    /// How many parallel tasks a group should have by default
    pub default_parallel_tasks: usize,
    /// Whether a group should be paused as soon as a single task fails
    pub pause_group_on_failure: bool,
    /// Whether the daemon (and all groups) should be paused as soon as a single task fails
    pub pause_all_on_failure: bool,
    /// The callback that's called whenever a task finishes.
    pub callback: Option<String>,
    /// The amount of log lines from stdout/stderr that are passed to the callback command.
    pub callback_log_lines: usize,
    /// This shouldn't be manipulated manually if the daemon is running.
    /// This represents all known groups and their amount of parallel tasks.
    pub groups: BTreeMap<String, usize>,
}

/// The parent settings struct. \
/// This contains all other setting structs.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub client: Client,
    pub daemon: Daemon,
    pub shared: Shared,
}

impl Shared {
    pub fn expand(old_path: &Path) -> PathBuf {
        PathBuf::from(tilde(&old_path.to_string_lossy()).into_owned())
    }

    pub fn pueue_directory(&self) -> PathBuf {
        Shared::expand(&self.pueue_directory)
    }

    #[cfg(not(target_os = "windows"))]
    pub fn unix_socket_path(&self) -> PathBuf {
        Shared::expand(&self.unix_socket_path)
    }

    pub fn daemon_cert(&self) -> PathBuf {
        Shared::expand(&self.daemon_cert)
    }
    pub fn daemon_key(&self) -> PathBuf {
        Shared::expand(&self.daemon_key)
    }
    pub fn shared_secret_path(&self) -> PathBuf {
        Shared::expand(&self.shared_secret_path)
    }
}

impl Settings {
    /// Read from existing config files.
    /// If no config files can be found or fields are missing, an error is returned.
    pub fn read(from_file: &Option<PathBuf>) -> Result<Settings, Error> {
        let config = Config::new();

        parse_config(config, true, from_file)
    }

    /// Try to read existing config files and
    /// This function creates a new configuration instance and
    /// populates it with default values for every option. \
    /// If a local config file already exists, it is parsed and
    /// overrules the default option values.
    ///
    /// The default local config is located at "~/.config/pueue.yml".
    ///
    /// If `require_config` is `true`, an error will be thrown, if no configuration file can be found.
    /// This is utilized by the client, since only the daemon is allowed to touch the configuration
    /// file.
    pub fn read_with_defaults(
        require_config: bool,
        from_file: &Option<PathBuf>,
    ) -> Result<Settings, Error> {
        let config = Settings::default_config()?;

        parse_config(config, require_config, from_file)
    }

    pub fn default_config() -> Result<Config, Error> {
        let mut config = Config::new();
        let pueue_path = default_pueue_path()?;
        config
            .set_default("shared.pueue_directory", pueue_path.clone())
            .unwrap();
        #[cfg(not(target_os = "windows"))]
        config.set_default("shared.use_unix_socket", true).unwrap();
        #[cfg(not(target_os = "windows"))]
        config
            .set_default("shared.unix_socket_path", get_unix_socket_path()?)
            .unwrap();

        config.set_default("shared.host", "127.0.0.1").unwrap();
        config.set_default("shared.port", "6924").unwrap();
        config.set_default("shared.tls_enabled", true).unwrap();
        config
            .set_default(
                "shared.daemon_key",
                pueue_path.clone() + "/certs/daemon.key",
            )
            .unwrap();
        config
            .set_default(
                "shared.daemon_cert",
                pueue_path.clone() + "/certs/daemon.cert",
            )
            .unwrap();
        config
            .set_default("shared.shared_secret_path", pueue_path + "/shared_secret")
            .unwrap();

        // Client specific config
        config.set_default("client.restart_in_place", true).unwrap();
        config.set_default("client.read_local_logs", true).unwrap();
        config
            .set_default("client.show_expanded_aliases", false)
            .unwrap();
        config
            .set_default("client.show_confirmation_questions", false)
            .unwrap();
        config.set_default("client.dark_mode", false).unwrap();
        config
            .set_default("client.max_status_lines", None::<i64>)
            .unwrap();
        config
            .set_default("client.newline_between_date_and_time", false)
            .unwrap();
        config
            .set_default("client.status_time_format", "%H:%M:%S".to_string())
            .unwrap();
        config
            .set_default(
                "client.status_datetime_format",
                "%Y-%m-%d\n%H:%M:%S".to_string(),
            )
            .unwrap();

        // Daemon specific config
        config
            .set_default("daemon.default_parallel_tasks", 1)
            .unwrap();
        config
            .set_default("daemon.pause_group_on_failure", false)
            .unwrap();
        config
            .set_default("daemon.pause_all_on_failure", false)
            .unwrap();
        config
            .set_default("daemon.callback", None::<String>)
            .unwrap();
        config.set_default("daemon.callback_log_lines", 10).unwrap();
        config
            .set_default("daemon.groups", HashMap::<String, i64>::new())
            .unwrap();

        Ok(config)
    }

    /// Save the current configuration as a file to the given path. \
    /// If no path is given, the default configuration path will be used. \
    /// The file is then written to the main configuration directory of the respective OS.
    pub fn save(&self, path: &Option<PathBuf>) -> Result<(), Error> {
        let config_path = if let Some(path) = path {
            path.clone()
        } else {
            default_config_directory()?.join("pueue.yml")
        };
        let config_dir = config_path
            .parent()
            .ok_or_else(|| Error::InvalidPath("Couldn't resolve config directory".into()))?;

        // Create the config dir, if it doesn't exist yet
        if !config_dir.exists() {
            create_dir_all(config_dir)?;
        }

        let content = match serde_yaml::to_string(self) {
            Ok(content) => content,
            Err(error) => {
                return Err(Error::Generic(format!(
                    "Configuration file serialization failed:\n{}",
                    error
                )))
            }
        };
        let mut file = File::create(config_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

/// Get all possible configuration paths and check if there are
/// configuration files at those locations.
/// All configs will be merged by importance.
///
/// If `require_config` is `true`, an error will be thrown, if no configuration file can be found.
fn parse_config(
    mut config: Config,
    require_config: bool,
    from_file: &Option<PathBuf>,
) -> Result<Settings, Error> {
    // Load the config from a very specific file path
    if let Some(path) = from_file {
        if !path.exists() || !path.is_file() {
            return Err(Error::FileNotFound(format!(
                "Couldn't find config at path {:?}",
                path
            )));
        }
        info!("Using config file at: {:?}", path);
        let config_file = config::File::with_name(path.to_str().unwrap());
        config.merge(config_file)?;
        return Ok(config.try_into()?);
    };

    let mut config_found = false;
    info!("Parsing config files");
    for directory in get_config_directories()?.into_iter() {
        let path = directory.join("pueue.yml");
        info!("Checking path: {:?}", &path);

        // Check if the file exists and parse it.
        if path.exists() && path.is_file() {
            info!("Found config file at: {:?}", path);
            config_found = true;
            let config_file = config::File::with_name(path.to_str().unwrap());
            config.merge(config_file)?;
        }
    }

    if require_config && !config_found {
        return Err(Error::FileNotFound(
            "Couldn't find a configuration file. Did you start the daemon yet?".into(),
        ));
    }

    // Try to can deserialize the entire configuration
    Ok(config.try_into()?)
}
