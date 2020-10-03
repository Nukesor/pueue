use ::std::process::Child;

use ::anyhow::Result;
use ::log::{debug, warn};
use ::nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use procfs::process::{all_processes, Process};

use crate::task_handler::ProcessAction;

/// Send a signal to one of Pueue's child process handles.
/// We need a special since there exists some inconsistent behavior.
///
/// In some circumstances and environments `sh -c $command` doesn't spawn a shell,
/// but rather spawns the `$command` directly.
///
/// This makes things a lot more complicated, since we need to either send signals
/// to the root process directly OR to all it's child processes.
/// This also affects the `--children` flag on all commands. We then have to either send the signal
/// to all direct children or to all of the childrens' children.
pub fn send_signal_to_child(
    child: &Child,
    action: &ProcessAction,
    send_to_children: bool,
) -> Result<bool> {
    let signal = get_signal_from_action(action);
    let pid = child.id() as i32;

    // Get the /proc representation of the child, so we can do some checks
    let process = if let Ok(process) = Process::new(pid) {
        process
    } else {
        // Process might have just gone away
        return Ok(false);
    };

    // Get the root command and, so we check whether it's actually a shell with `sh -c`.
    let mut cmdline = if let Ok(cmdline) = process.cmdline() {
        cmdline
    } else {
        // Process might have just gone away
        return Ok(false);
    };

    // Now we know whether this is a directly spawned process or a process wrapped by a shell.
    let is_shell = is_cmdline_shell(&mut cmdline);

    if is_shell {
        // If it's a shell, we have to send the signal to the actual shell and to all it's children.
        // There might be multiple children, for instance, when users use the `&` operator.
        // If the `send_to_children` flag is given, the

        // Send the signal to the shell, don't propagate to it's children yet.
        send_signal_to_process(pid, action, false)?;

        // Now send the signal to the shells child processes and their respective
        // children if the user wants to do so.
        let shell_children = get_child_processes(pid).unwrap();
        for shell_child in shell_children {
            send_signal_to_process(shell_child.pid(), action, send_to_children)?;
        }
    } else {
        // If it isn't a shell, send the signal directly to the process.
        // Handle children normally.
        send_signal_to_process(pid, action, send_to_children)?;
    }

    signal::kill(Pid::from_raw(pid), signal)?;
    Ok(true)
}

/// Check whether a process's commandline string is actually a shell or not
pub fn is_cmdline_shell(cmdline: &mut Vec<String>) -> bool {
    if cmdline.len() < 3 {
        return false;
    }

    if cmdline.remove(0) != "sh" {
        return false;
    }

    if cmdline.remove(0) != "-c" {
        return false;
    }

    true
}

/// Send a signal to a unix process.
pub fn send_signal_to_process(
    pid: i32,
    action: &ProcessAction,
    send_to_children: bool,
) -> Result<bool, nix::Error> {
    let signal = get_signal_from_action(action);
    debug!("Sending signal {} to {}", signal, pid);

    // Send the signal to all children, if that's what the user wants.
    if send_to_children {
        send_signal_to_children(pid, action);
    }

    signal::kill(Pid::from_raw(pid), signal)?;
    Ok(true)
}

/// A small helper that sends a signal to all children of a specific process by id.
pub fn send_signal_to_children(pid: i32, action: &ProcessAction) {
    send_signal_to_processes(get_child_processes(pid).unwrap(), action);
}

fn get_signal_from_action(action: &ProcessAction) -> Signal {
    match action {
        ProcessAction::Kill => Signal::SIGKILL,
        ProcessAction::Pause => Signal::SIGSTOP,
        ProcessAction::Resume => Signal::SIGCONT,
    }
}

/// Get all children of a specific process
pub fn get_child_processes(pid: i32) -> Option<Vec<Process>> {
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
