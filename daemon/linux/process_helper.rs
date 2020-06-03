use ::log::warn;
use ::nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use procfs::process::{all_processes, Process};

/// Get all children of a specific process
pub fn get_children(pid: i32) -> Vec<Process> {
    all_processes()
        .unwrap()
        .into_iter()
        .filter(|process| process.stat.ppid == pid)
        .collect()
}

/// Send a signal to a list of processes
pub fn send_signal_to_processes(processes: Vec<Process>, signal: Signal) {
    for process in processes {
        // Process is no longer alive, skip this.
        if !process.is_alive() {
            continue;
        }

        let pid = Pid::from_raw(process.pid);
        if let Err(error) = signal::kill(pid, signal) {
            warn!(
                "Failed send signal {:?} to Pid {}: {:?}",
                signal, process.pid, error
            );
        }
    }
}

/// A small helper that sends a signal to all children of a specific process by id.
pub fn send_signal_to_children(pid: i32, signal: Signal) {
    send_signal_to_processes(get_children(pid), signal);
}
