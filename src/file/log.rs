use ::failure::Error;
use ::std::fs::File;

pub fn open_log_file_handles(index: usize) -> Result<(File, File), Error> {
    let stdout_log = File::open(format!("{}_stdout.log", index))?;
    let stderr_log = File::open(format!("{}_stderr.log", index))?;

    Ok((stdout_log, stderr_log))
}

pub fn create_log_file_handles(index: usize) -> Result<(File, File), Error> {
    let stdout_log = File::create(format!("{}_stdout.log", index))?;
    let stderr_log = File::create(format!("{}_stderr.log", index))?;

    Ok((stdout_log, stderr_log))
}
