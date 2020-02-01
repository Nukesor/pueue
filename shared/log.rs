use ::anyhow::Result;
use ::byteorder::{LittleEndian, ReadBytesExt};
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

/// Convert stdout or stderr data from a spawned task to a string.
pub fn process_output_to_text(mut data: &[u8]) -> String {
    if cfg!(windows) {
        // On windows we run the command using "cmd" with a flag that makes it output Unicode (UTF16).

        let is_odd = data.len() % 2 == 1;
        let mut buffer = Vec::with_capacity(data.len() / 2 + if is_odd { 1 } else { 0 });

        type CmdUtf16Endian = LittleEndian;

        while data.len() > 1 {
            buffer.push(data.read_u16::<CmdUtf16Endian>().unwrap());
        }
        if is_odd {
            let extra = [*data.last().unwrap(), 0];
            let mut extra = &extra as &[u8];
            buffer.push(extra.read_u16::<CmdUtf16Endian>().unwrap());
        }

        String::from_utf16_lossy(&buffer)
    } else {
        String::from_utf8_lossy(data).into_owned()
    }
}

/// Return the content of temporary stdout and stderr files for a task
pub fn read_log_files(task_id: usize, settings: &Settings) -> Result<(String, String)> {
    let (mut stdout_handle, mut stderr_handle) = get_log_file_handles(task_id, settings)?;
    let mut stdout_buffer = Vec::new();
    let mut stderr_buffer = Vec::new();

    stdout_handle.read_to_end(&mut stdout_buffer)?;
    stderr_handle.read_to_end(&mut stderr_buffer)?;

    let stdout = process_output_to_text(&stdout_buffer);
    let stderr = process_output_to_text(&stderr_buffer);

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
