use std::fs::{read_dir, remove_file, File};
use std::io::{self, BufReader, Cursor};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use log::error;
use snap::write::FrameEncoder;

/// Return the paths to the `(stdout, stderr)` log files of a task.
pub fn get_log_paths(task_id: usize, path: &Path) -> (PathBuf, PathBuf) {
    let task_log_dir = path.join("task_logs");
    let out_path = task_log_dir.join(format!("{}_stdout.log", task_id));
    let err_path = task_log_dir.join(format!("{}_stderr.log", task_id));
    (out_path, err_path)
}

/// Create and return the file handle for the `(stdout, stderr)` log files of a task.
pub fn create_log_file_handles(task_id: usize, path: &Path) -> Result<(File, File)> {
    let (out_path, err_path) = get_log_paths(task_id, path);
    let stdout = File::create(out_path)?;
    let stderr = File::create(err_path)?;

    Ok((stdout, stderr))
}

/// Return the file handle for the `(stdout, stderr)` log files of a task.
pub fn get_log_file_handles(task_id: usize, path: &Path) -> Result<(File, File)> {
    let (out_path, err_path) = get_log_paths(task_id, path);
    let stdout = File::open(out_path)?;
    let stderr = File::open(err_path)?;

    Ok((stdout, stderr))
}

/// Remove the the log files of a task.
pub fn clean_log_handles(task_id: usize, path: &Path) {
    let (out_path, err_path) = get_log_paths(task_id, path);
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

/// Return the `(stdout, stderr)` output of a task. \
/// Task output is compressed using [snap] to save some memory and bandwidth.
pub fn read_and_compress_log_files(
    task_id: usize,
    path: &Path,
    lines: Option<usize>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let (mut stdout_file, mut stderr_file) = match get_log_file_handles(task_id, path) {
        Ok((stdout, stderr)) => (stdout, stderr),
        Err(err) => {
            bail!("Error while opening the output files: {}", err);
        }
    };

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    if let Some(lines) = lines {
        // Get the last few lines of both files
        let stdout_bytes = read_last_lines(&mut stdout_file, lines).into_bytes();
        let stderr_bytes = read_last_lines(&mut stderr_file, lines).into_bytes();
        let mut stdout_cursor = Cursor::new(stdout_bytes);
        let mut stderr_cursor = Cursor::new(stderr_bytes);

        // Compress the partial log input and pipe it into the snappy compressor
        let mut stdout_compressor = FrameEncoder::new(&mut stdout);
        io::copy(&mut stdout_cursor, &mut stdout_compressor)?;
        let mut stderr_compressor = FrameEncoder::new(&mut stderr);
        io::copy(&mut stderr_cursor, &mut stderr_compressor)?;
    } else {
        // Compress the full log input and pipe it into the snappy compressor
        let mut stdout_compressor = FrameEncoder::new(&mut stdout);
        io::copy(&mut stdout_file, &mut stdout_compressor)?;
        let mut stderr_compressor = FrameEncoder::new(&mut stderr);
        io::copy(&mut stderr_file, &mut stderr_compressor)?;
    }

    Ok((stdout, stderr))
}

/// Return the last lines of `(stdout, stderr)` of a task. \
/// This output is uncompressed and may take a lot of memory, which is why we only read
/// the last few lines.
pub fn read_last_log_file_lines(
    task_id: usize,
    path: &Path,
    lines: usize,
) -> Result<(String, String)> {
    let (mut stdout_file, mut stderr_file) = match get_log_file_handles(task_id, path) {
        Ok((stdout, stderr)) => (stdout, stderr),
        Err(err) => {
            bail!(
                "Error while opening the log files for task {}: {}",
                task_id,
                err
            );
        }
    };

    // Get the last few lines of both files
    Ok((
        read_last_lines(&mut stdout_file, lines),
        read_last_lines(&mut stderr_file, lines),
    ))
}

/// Remove all files in the log directory.
pub fn reset_task_log_directory(path: &Path) {
    let task_log_dir = path.join("task_logs");

    let files = read_dir(task_log_dir).expect("Failed to open pueue's task_logs directory");

    for file in files.flatten() {
        if let Err(err) = remove_file(file.path()) {
            error!("Failed to delete log file: {}", err);
        }
    }
}

/// Read the last `amount` lines of a file to a string.
///
/// TODO: This is super imperformant, but works as long as we don't use the last
/// 1000 lines. It would be cleaner to seek to the beginning of the requested
/// position and simply stream the content.
pub fn read_last_lines(file: &mut File, amount: usize) -> String {
    let last_lines: Vec<String> = rev_lines::RevLines::new(BufReader::new(file))
        .expect("Failed to read last lines of file")
        .take(amount)
        .collect();

    last_lines
        .into_iter()
        .rev()
        .collect::<Vec<String>>()
        .join("\n")
}
