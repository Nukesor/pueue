use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use log::{info, warn};

use crate::error::Error;
use crate::platform::directories::get_config_directories;

/// Return the contents of the alias file, if it exists and can be parsed. \
/// The file has to be located in `pueue_directory` and named `pueue_aliases.yml`.
pub fn get_aliases() -> Result<HashMap<String, String>, Error> {
    // Go through all config directories and check for a alias file.
    let mut alias_file_path = None;
    for directory in get_config_directories() {
        let path = directory.join("pueue_aliases.yml");
        if path.exists() {
            alias_file_path = Some(path);
        }
    }

    // Return early if we cannot find the file
    let path = match alias_file_path {
        None => {
            info!("Didn't find pueue alias file.");
            return Ok(HashMap::new());
        }
        Some(alias_file_path) => alias_file_path,
    };

    // Read the file content
    let mut alias_file = File::open(&path)
        .map_err(|err| Error::IoPathError(path.clone(), "opening alias file", err))?;
    let mut content = String::new();
    alias_file
        .read_to_string(&mut content)
        .map_err(|err| Error::IoPathError(path.clone(), "reading alias file", err))?;

    serde_yaml::from_str(&content).map_err(|err| {
        Error::ConfigDeserialization(format!("Failed to read alias configuration file:\n{err}"))
    })
}

/// Check if there exists an alias for a given command.
/// Only the first word will be replaced.
pub fn insert_alias(command: String) -> String {
    let first = match command.split_whitespace().next() {
        Some(first) => first,
        None => return command,
    };

    let aliases = match get_aliases() {
        Err(err) => {
            warn!("Failed to open aliases file: {err}");
            return command;
        }
        Ok(aliases) => aliases,
    };

    for (original, alias) in aliases.iter() {
        if original == first {
            return command.replacen(original, alias, 1);
        }
    }

    command
}
