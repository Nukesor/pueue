use ::log::warn;
use ::nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use psutil::process::processes;

/// A small helper to send a signal to all direct child processes of a specific task.
pub fn send_signal_to_children(pid: i32, signal: Signal) {
    let all_processes = match processes() {
        Err(error) => {
            warn!("Failed to get full process list: {}", error);
            return;
        }
        Ok(processes) => processes,
    };
    let children = all_processes
        .into_iter()
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .filter(|process| process.ppid() as i32 == pid);

    for child in children {
        let pid = Pid::from_raw(child.pid() as i32);

        if let Err(error) = signal::kill(pid, signal) {
            warn!(
                "Failed send signal {:?} to Pid {}: {:?}",
                signal,
                child.pid(),
                error
            );
        }
    }
}
