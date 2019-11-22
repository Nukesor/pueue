use ::anyhow::Result;
use ::log::error;
use ::std::fs::{remove_file, File};
use ::std::io::prelude::*;

pub fn create_log_file_handles(task_id: i32) -> Result<(File, File)> {
    let stdout_file = File::create(format!("/tmp/{}_stdout.log", task_id))?;
    let stderr_file = File::create(format!("/tmp/{}_stderr.log", task_id))?;

    Ok((stdout_file, stderr_file))
}

pub fn read_log_files(task_id: i32) -> Result<(String, String)> {
    let mut stdout_file = File::open(format!("/tmp/{}_stdout.log", task_id))?;
    let mut stderr_file = File::open(format!("/tmp/{}_stderr.log", task_id))?;

    let mut stdout = String::new();
    let mut stderr = String::new();

    stdout_file.read_to_string(&mut stdout)?;
    stderr_file.read_to_string(&mut stderr)?;

    Ok((stdout, stderr))
}

pub fn clean_log_handles(task_id: i32) {
    if let Err(err) = remove_file(format!("/tmp/{}_stdout.log", task_id)) {
        error!(
            "Failed to remove stdout file for task {} with error {:?}",
            task_id, err
        );
    };
    if let Err(err) = remove_file(format!("/tmp/{}_stderr.log", task_id)) {
        error!(
            "Failed to remove stderr file for task {} with error {:?}",
            task_id, err
        );
    };
}
