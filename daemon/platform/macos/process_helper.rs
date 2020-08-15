use log::warn;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use psutil::process::{processes, Process};

use crate::task_handler::ProcessAction;

/// Send a signal to a unix process.
pub fn send_signal(pid: u32, action: &ProcessAction, children: bool) -> Result<bool, nix::Error> {
    let signal = get_signal_from_action(action);
    debug!("Sending signal {} to {}", signal, pid);

    // Send the signal to all children, if that's what the user wants.
    if children {
        send_signal_to_children(pid as i32, action);
    }

    signal::kill(Pid::from_raw(pid as i32), signal)?;
    Ok(true)
}

/// A small helper to send a signal to all direct child processes of a specific task.
pub fn send_signal_to_children(pid: i32, action: &ProcessAction) {
    send_signal_to_processes(get_children(pid).unwrap(), action);
}

fn get_signal_from_action(action: &ProcessAction) -> Signal {
    match action {
        ProcessAction::Kill => Signal::SIGKILL,
        ProcessAction::Pause => Signal::SIGSTOP,
        ProcessAction::Resume => Signal::SIGCONT,
    }
}

/// Get all children of a specific process
pub fn get_children(pid: i32) -> Option<Vec<Process>> {
    let all_processes = match processes() {
        Err(error) => {
            warn!("Failed to get full process list: {}", error);
            return Vec::new();
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
        .collect();

    Some(all_processes)
}

/// Send a signal to a list of processes
pub fn send_signal_to_processes(processes: Vec<Process>, action: &ProcessAction) {
    let signal = get_signal_from_action(action);
    for process in processes {
        let pid = Pid::from_raw(process.pid() as i32);

        // Process is no longer alive, skip this.
        if !process.is_running() {
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
