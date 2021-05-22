use async_std::task;
use nix::sys::signal;

use pueue_daemon_lib::run;

mod helper;

#[async_std::test]
/// Spin up the daemon and send a SIGTERM shortly afterwards.
/// This should trigger
async fn test_ctrlc() {
    let (_settings, tempdir) = helper::get_settings();

    // Start/spin off the daemon and get its PID
    task::spawn(run(Some(tempdir.path().join("pueue.yml"))));
    let pid = helper::get_pid(tempdir.path());

    // Wait a little for the daemon to properly start
    helper::sleep_ms(500);

    // Send SIGTERM signal to process via nix
    let nix_pid = nix::unistd::Pid::from_raw(pid);
    signal::kill(nix_pid, signal::Signal::SIGTERM).expect("Failed to send SIGTERM to pid");

    // Sleep for 500ms and give the daemon time to shut down
    helper::sleep_ms(500);

    // Get all processes and make sure the process with our pid no longer exists
    let processes = procfs::process::all_processes().expect("Failed to get all processes");
    assert!(processes
        .iter()
        .filter(|process| process.pid == pid)
        .collect::<Vec<_>>()
        .is_empty());
}
