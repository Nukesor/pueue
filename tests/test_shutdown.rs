use anyhow::{Context, Result};
mod helper;

use helper::*;

#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Spin up the daemon and send a SIGTERM shortly afterwards.
/// This should trigger the graceful shutdown and kill the process.
async fn test_ctrlc() -> Result<()> {
    let (_settings, tempdir) = helper::base_setup()?;

    // Same as boot_daemon, but starts the daemon in non test mode
    tokio::spawn(run_and_handle_error(
        tempdir.path().clone().to_path_buf(),
        false,
    ));
    let pid = get_pid(tempdir.path())?;

    // Send SIGTERM signal to process via nix
    use nix::sys::signal;
    let nix_pid = nix::unistd::Pid::from_raw(pid);
    signal::kill(nix_pid, signal::Signal::SIGTERM).context("Failed to send SIGTERM to pid")?;

    // Sleep for 500ms and give the daemon time to shut down
    helper::sleep_ms(500);

    // Get all processes and make sure the process with our pid no longer exists
    // However, since the daemon shuts down gracefully on SIGTERM, it'll exit the test.
    // This is why the following code will never be reached or rather, if it will be reached, it'll
    // fail.
    let processes = procfs::process::all_processes().context("Failed to get all processes")?;
    assert!(processes
        .iter()
        .filter(|process| process.pid == pid)
        .collect::<Vec<_>>()
        .is_empty());

    Ok(())
}
