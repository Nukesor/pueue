use ::log::warn;
use ::nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use procfs::process::all_processes;

/// A small helper to send a signal to all direct child processes of a specific task.
pub fn send_signal_to_children(pid: i32, signal: Signal) {
    let children = all_processes()
        .unwrap()
        .into_iter()
        .filter(|process| process.stat.ppid == pid);

    for child in children {
        let pid = Pid::from_raw(child.pid);
        if let Err(error) = signal::kill(pid, signal) {
            warn!(
                "Failed send signal {:?} to Pid {}: {:?}",
                signal, child.pid, error
            );
        }
    }
}
