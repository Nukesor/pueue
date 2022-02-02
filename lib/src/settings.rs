use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};

use log::info;
use serde_derive::{Deserialize, Serialize};
use shellexpand::tilde;

use crate::error::Error;
use crate::platform::directories::*;
use crate::setting_defaults::*;

/// All settings which are used by both, the client and the daemon
#[derive(PartialEq, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Shared {
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The directory that is used for all of pueue's state. \
    /// I.e. task logs, state dumps, etc.
    pub pueue_directory: Option<PathBuf>,
    /// The location where runtime related files will be placed.
    /// Defaults to `pueue_directory` unless `$XDG_RUNTIME_DIR` is set.
    pub runtime_directory: Option<PathBuf>,
    /// If this is set to true, unix sockets will be used.
    /// Otherwise we default to TCP+TLS
    #[cfg(not(target_os = "windows"))]
    #[serde(default = "default_true")]
    pub use_unix_socket: bool,
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The path to the unix socket.
    #[cfg(not(target_os = "windows"))]
    pub unix_socket_path: Option<PathBuf>,

    /// The TCP hostname/ip address.
    #[serde(default = "default_host")]
    pub host: String,
    /// The TCP port.
    #[serde(default = "default_port")]
    pub port: String,
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The path to the TLS certificate used by the daemon. \
    /// This is also used by the client to verify the daemon's identity.
    pub daemon_cert: Option<PathBuf>,
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The path to the TLS key used by the daemon.
    pub daemon_key: Option<PathBuf>,
    /// Don't access this property directly, but rather use the getter with the same name.
    /// It's only public to allow proper integration testing.
    ///
    /// The path to the file containing the shared secret used to authenticate the client.
    pub shared_secret_path: Option<PathBuf>,
}

/// All settings which are used by the client
#[derive(PartialEq, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Client {
    /// If set to true, all tasks will be restart in place, instead of creating a new task.
    /// False is the default, as you'll lose the logs of the previously failed tasks when
    /// restarting tasks in place.
    #[serde(default = "Default::default")]
    pub restart_in_place: bool,
    /// Whether the client should read the logs directly from disk or whether it should
    /// request the data from the daemon via socket.
    #[serde(default = "default_true")]
    pub read_local_logs: bool,
    /// Whether the client should show a confirmation question on potential dangerous actions.
    #[serde(default = "Default::default")]
    pub show_confirmation_questions: bool,
    /// Whether aliases specified in `pueue_aliases.yml` should be expanded in the `pueue status`
    /// or shown in their short form.
    #[serde(default = "Default::default")]
    pub show_expanded_aliases: bool,
    /// Whether the client should use dark shades instead of regular colors.
    #[serde(default = "Default::default")]
    pub dark_mode: bool,
    /// The max amount of lines each task get's in the `pueue status` view.
    pub max_status_lines: Option<usize>,
    /// The format that will be used to display time formats in `pueue status`.
    #[serde(default = "default_status_time_format")]
    pub status_time_format: String,
    /// The format that will be used to display datetime formats in `pueue status`.
    #[serde(default = "default_status_datetime_format")]
    pub status_datetime_format: String,
}

/// All settings which are used by the daemon
#[derive(PartialEq, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Daemon {
    /// Whether a group should be paused as soon as a single task fails
    #[serde(default = "Default::default")]
    pub pause_group_on_failure: bool,
    /// Whether the daemon (and all groups) should be paused as soon as a single task fails
    #[serde(default = "Default::default")]
    pub pause_all_on_failure: bool,
    /// The callback that's called whenever a task finishes.
    pub callback: Option<String>,
    /// The amount of log lines from stdout/stderr that are passed to the callback command.
    #[serde(default = "default_callback_log_lines")]
    pub callback_log_lines: usize,
    /// The legacy configuration for groups
    #[serde(skip_serializing)]
    #[deprecated(
        since = "1.1.0",
        note = "The configuration for groups is now stored in the state."
    )]
    pub groups: Option<HashMap<String, i64>>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            client: Client {
                read_local_logs: true,
                status_time_format: default_status_time_format(),
                status_datetime_format: default_status_datetime_format(),
                ..Default::default()
            },
            daemon: Daemon {
                callback_log_lines: default_callback_log_lines(),
                ..Default::default()
            },
            shared: Shared {
                #[cfg(not(target_os = "windows"))]
                use_unix_socket: true,
                host: default_host(),
                port: default_port(),
                ..Default::default()
            },
            profiles: HashMap::new(),
        }
    }
}

/// The parent settings struct. \
/// This contains all other setting structs.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default = "Default::default")]
    pub client: Client,
    #[serde(default = "Default::default")]
    pub daemon: Daemon,
    pub shared: Shared,
    #[serde(default = "HashMap::new")]
    pub profiles: HashMap<String, NestedSettings>,
}

/// The nested settings struct for profiles. \
/// In contrast to the normal `Settings` struct, this struct doesn't allow profiles.
/// That way we prevent nested profiles and problems with self-referencing structs.
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct NestedSettings {
    #[serde(default = "Default::default")]
    pub client: Client,
    #[serde(default = "Default::default")]
    pub daemon: Daemon,
    #[serde(default = "Default::default")]
    pub shared: Shared,
}

/// Little helper which expands a given path's `~` characters to a fully qualified path.
pub fn expand_home(old_path: &Path) -> PathBuf {
    PathBuf::from(tilde(&old_path.to_string_lossy()).into_owned())
}

impl Shared {
    pub fn pueue_directory(&self) -> PathBuf {
        if let Some(path) = &self.pueue_directory {
            expand_home(path)
        } else {
            default_pueue_path()
        }
    }

    /// Get the current runtime directory in the following precedence.
    /// 1. Config value
    /// 2. Environment configuration
    /// 3. Pueue directory
    pub fn runtime_directory(&self) -> PathBuf {
        if let Some(path) = &self.runtime_directory {
            expand_home(path)
        } else if let Some(path) = default_runtime_directory() {
            path
        } else {
            default_pueue_path()
        }
    }

    /// The unix socket path can either be explicitly specified or it's simply placed in the
    /// current runtime directory.
    #[cfg(not(target_os = "windows"))]
    pub fn unix_socket_path(&self) -> PathBuf {
        if let Some(path) = &self.unix_socket_path {
            expand_home(path)
        } else {
            self.runtime_directory()
                .join(format!("pueue_{}.socket", whoami::username()))
        }
    }

    pub fn daemon_cert(&self) -> PathBuf {
        if let Some(path) = &self.daemon_cert {
            expand_home(path)
        } else {
            self.pueue_directory().join("certs").join("daemon.cert")
        }
    }

    pub fn daemon_key(&self) -> PathBuf {
        if let Some(path) = &self.daemon_key {
            expand_home(path)
        } else {
            self.pueue_directory().join("certs").join("daemon.key")
        }
    }

    pub fn shared_secret_path(&self) -> PathBuf {
        if let Some(path) = &self.shared_secret_path {
            expand_home(path)
        } else {
            self.pueue_directory().join("shared_secret")
        }
    }
}

impl Settings {
    /// Try to read existing config files, while using default values for non-existing fields.
    /// If successful, this will return a full config as well as a boolean on whether we found an
    /// existing configuration file or not.
    ///
    /// The default local config locations depends on the current target.
    pub fn read(from_file: &Option<PathBuf>) -> Result<(Settings, bool), Error> {
        // Load the config from a very specific file path
        if let Some(path) = from_file {
            if !path.exists() || !path.is_file() {
                return Err(Error::FileNotFound(format!(
                    "Couldn't find config at path {path:?}"
                )));
            }

            // Open the file in read-only mode with buffer.
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            let settings = serde_yaml::from_reader(reader)
                .map_err(|err| Error::ConfigDeserialization(err.to_string()))?;
            return Ok((settings, true));
        };

        info!("Parsing config files");
        for directory in get_config_directories().into_iter() {
            let path = directory.join("pueue.yml");
            info!("Checking path: {path:?}");

            // Check if the file exists and parse it.
            if path.exists() && path.is_file() {
                info!("Found config file at: {path:?}");

                // Open the file in read-only mode with buffer.
                let file = File::open(path)?;
                let reader = BufReader::new(file);

                let settings = serde_yaml::from_reader(reader)
                    .map_err(|err| Error::ConfigDeserialization(err.to_string()))?;
                return Ok((settings, true));
            }
        }

        info!("No config file found. Use default config.");
        // Return a default configuration if we couldn't find a file.
        Ok((Settings::default(), false))
    }

    /// Save the current configuration as a file to the given path. \
    /// If no path is given, the default configuration path will be used. \
    /// The file is then written to the main configuration directory of the respective OS.
    pub fn save(&self, path: &Option<PathBuf>) -> Result<(), Error> {
        let config_path = if let Some(path) = path {
            path.clone()
        } else {
            default_config_directory().join("pueue.yml")
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
                    "Configuration file serialization failed:\n{error}"
                )))
            }
        };
        let mut file = File::create(config_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Try to load a profile. Error if it doesn't exist.
    pub fn load_profile(&mut self, profile: &str) -> Result<(), Error> {
        let profile = self.profiles.remove(profile).ok_or_else(|| {
            Error::ConfigDeserialization(format!("Couldn't find profile with name \"{profile}\""))
        })?;

        self.client = profile.client;
        self.daemon = profile.daemon;
        self.shared = profile.shared;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Check if profiles get loaded correctly.
    #[test]
    fn test_load_profile() {
        // Create some default settings and ensure that default values are loaded.
        let mut settings = Settings::default();
        assert_eq!(
            settings.client.status_time_format,
            default_status_time_format()
        );
        assert_eq!(
            settings.daemon.callback_log_lines,
            default_callback_log_lines()
        );
        assert_eq!(settings.shared.host, default_host());

        // Crate a new profile with slightly different values.
        let mut profile = Settings::default();
        profile.client.status_time_format = "test".to_string();
        profile.daemon.callback_log_lines = 100_000;
        profile.shared.host = "quatschhost".to_string();
        let profile = NestedSettings {
            client: profile.client,
            daemon: profile.daemon,
            shared: profile.shared,
        };

        settings.profiles.insert("testprofile".to_string(), profile);

        // Load the profile and ensure the new values are now loaded.
        settings
            .load_profile("testprofile")
            .expect("We just added the profile");

        assert_eq!(settings.client.status_time_format, "test");
        assert_eq!(settings.daemon.callback_log_lines, 100_000);
        assert_eq!(settings.shared.host, "quatschhost");
    }

    /// A proper pueue [Error] should be thrown if the profile cannot be found.
    #[test]
    fn test_error_on_missing_profile() {
        let mut settings = Settings::default();

        let result = settings.load_profile("doesn't exist");
        let expected_error_message = "Couldn't find profile with name \"doesn't exist\"";
        if let Err(Error::ConfigDeserialization(error_message)) = result {
            assert_eq!(error_message, expected_error_message);
            return;
        }

        panic!("Got unexpected result when expecting missing profile error: {result:?}");
    }
}
