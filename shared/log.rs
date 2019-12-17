use ::anyhow::Result;
use ::log::error;
use ::std::fs::{remove_file, File};
use ::std::io::prelude::*;
use ::std::path::{Path, PathBuf};

use crate::settings::Settings;

/// Return the paths to temporary stdout and stderr files for a task
pub fn get_log_paths(task_id: usize, settings: &Settings) -> (PathBuf, PathBuf) {
    let pueue_dir = Path::new(&settings.daemon.pueue_directory).join("temp");
    let out_path = pueue_dir.join(format!("{}_stdout.log", task_id));
    let err_path = pueue_dir.join(format!("{}_stderr.log", task_id));
    (out_path, err_path)
}

/// Create and return the file handle for temporary stdout and stderr files for a task
pub fn create_log_file_handles(task_id: usize, settings: &Settings) -> Result<(File, File)> {
    let (out_path, err_path) = get_log_paths(task_id, settings);
    let stdout = File::create(out_path)?;
    let stderr = File::create(err_path)?;

    Ok((stdout, stderr))
}

/// Return the file handle for temporary stdout and stderr files for a task
pub fn get_log_file_handles(task_id: usize, settings: &Settings) -> Result<(File, File)> {
    let (out_path, err_path) = get_log_paths(task_id, settings);
    let stdout = File::open(out_path)?;
    let stderr = File::open(err_path)?;

    Ok((stdout, stderr))
}

/// Return the content of temporary stdout and stderr files for a task
pub fn read_log_files(task_id: usize, settings: &Settings) -> Result<(String, String)> {
    let (mut stdout_handle, mut stderr_handle) = get_log_file_handles(task_id, settings)?;
    let mut stdout = String::new();
    let mut stderr = String::new();

    stdout_handle.read_to_string(&mut stdout)?;
    stderr_handle.read_to_string(&mut stderr)?;

    Ok((stdout, stderr))
}

/// Remove temporary stdout and stderr files for a task
pub fn clean_log_handles(task_id: usize, settings: &Settings) {
    let (out_path, err_path) = get_log_paths(task_id, settings);
    if let Err(err) = remove_file(out_path) {
        error!(
            "Failed to remove stdout file for task {} with error {:?}",
            task_id, err
        );
    };
    if let Err(err) = remove_file(err_path) {
        error!(
            "Failed to remove stderr file for task {} with error {:?}",
            task_id, err
        );
    };
}
