use ::log::{debug, warn};
use ::nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use procfs::process::{all_processes, Process};

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

/// A small helper that sends a signal to all children of a specific process by id.
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
    Some(
        all_processes()
            .unwrap()
            .into_iter()
            .filter(|process| process.stat.ppid == pid)
            .collect(),
    )
}

/// Send a signal to a list of processes
pub fn send_signal_to_processes(processes: Vec<Process>, action: &ProcessAction) {
    let signal = get_signal_from_action(action);
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
