use ::failure::Error;
use ::std::fs::File;

pub fn open_log_file_handles(index: usize) -> Result<(File, File), Error> {
    let stdout_log = File::open(format!("/tmp/{}_stdout.log", index))?;
    let stderr_log = File::open(format!("/tmp/{}_stderr.log", index))?;

    Ok((stdout_log, stderr_log))
}

pub fn create_log_file_handles(index: usize) -> Result<(File, File), Error> {
    let stdout_log = File::create(format!("/tmp/{}_stdout.log", index))?;
    let stderr_log = File::create(format!("/tmp/{}_stderr.log", index))?;

    Ok((stdout_log, stderr_log))
}
