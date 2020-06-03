use ::log::warn;
use ::nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use psutil::process::{processes, Process};

/// Get all children of a specific process
pub fn get_children(pid: i32) -> Vec<Process> {
    let all_processes = match processes() {
        Err(error) => {
            warn!("Failed to get full process list: {}", error);
            return;
        }
        Ok(processes) => processes,
    };
    all_processes
        .into_iter()
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .filter(|process| {
            if let Ok(ppid) = process.ppid() {
                if let Some(ppid) = ppid {
                    return ppid as i32 == pid;
                }
            }
            false
        })
}

/// Send a signal to a list of processes
pub fn send_signal_to_processes(processes: Vec<Process>, signal: Signal) {
    for process in processes {
        let pid = Pid::from_raw(process.pid() as i32);

        // Process is no longer alive, skip this.
        if !process.is_running {
            continue;
        }

        if let Err(error) = signal::kill(pid, signal) {
            warn!(
                "Failed send signal {:?} to Pid {}: {:?}",
                signal,
                process.pid(),
                error
            );
        }
    }
}

/// A small helper to send a signal to all direct child processes of a specific task.
pub fn send_signal_to_children(pid: i32, signal: Signal) {
    send_signal_to_processes(get_children(pid), signal);
}
