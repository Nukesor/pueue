use anyhow::{Context, Result};
use nix::sys::signal;

mod helper;

#[async_std::test]
/// Spin up the daemon and send a SIGTERM shortly afterwards.
/// This should trigger
async fn test_ctrlc() -> Result<()> {
    let (_settings, tempdir) = helper::base_setup()?;

    let pid = helper::start_daemon(tempdir.path())?;

    // Send SIGTERM signal to process via nix
    let nix_pid = nix::unistd::Pid::from_raw(pid);
    signal::kill(nix_pid, signal::Signal::SIGTERM).context("Failed to send SIGTERM to pid")?;

    // Sleep for 500ms and give the daemon time to shut down
    helper::sleep_ms(500);

    // Get all processes and make sure the process with our pid no longer exists
    let processes = procfs::process::all_processes().context("Failed to get all processes")?;
    assert!(processes
        .iter()
        .filter(|process| process.pid == pid)
        .collect::<Vec<_>>()
        .is_empty());

    Ok(())
}
