use std::convert::TryInto;
use std::process::{Child, Command};

use anyhow::Result;
use log::debug;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};

use crate::task_handler::ProcessAction;
use pueue_lib::network::message::Signal as InternalSignal;

pub fn compile_shell_command(command_string: &str) -> Command {
    let mut command = Command::new("sh");
    command.arg("-c").arg(command_string);

    command
}

fn map_action_to_signal(action: &ProcessAction) -> Signal {
    match action {
        ProcessAction::Pause => Signal::SIGSTOP,
        ProcessAction::Resume => Signal::SIGCONT,
    }
}

fn map_internal_signal_to_nix_signal(signal: InternalSignal) -> Signal {
    match signal {
        InternalSignal::SigKill => Signal::SIGKILL,
        InternalSignal::SigInt => Signal::SIGINT,
        InternalSignal::SigTerm => Signal::SIGTERM,
        InternalSignal::SigCont => Signal::SIGCONT,
        InternalSignal::SigStop => Signal::SIGSTOP,
    }
}

/// Convenience wrapper around `send_signal_to_child` for raw unix signals.
/// Its purpose is to hide platform specific logic.
pub fn send_internal_signal_to_child(
    child: &Child,
    signal: InternalSignal,
    send_to_children: bool,
) -> Result<bool> {
    let signal = map_internal_signal_to_nix_signal(signal);
    send_signal_to_child(child, signal, send_to_children)
}

/// Convenience wrapper around `send_signal_to_child` for internal actions on processes.
/// Its purpose is to hide platform specific logic.
pub fn run_action_on_child(
    child: &Child,
    action: &ProcessAction,
    send_to_children: bool,
) -> Result<bool> {
    let signal = map_action_to_signal(action);
    send_signal_to_child(child, signal, send_to_children)
}

/// Send a signal to one of Pueue's child process handles.
/// We need a special since we assume that there's also a `sh -c` around the actuall process.
pub fn send_signal_to_child(
    child: &Child,
    signal: Signal,
    _send_to_children: bool,
) -> Result<bool> {
    let pid = child.id();
    // Send the signal to the shell, don't propagate to its children yet.
    send_signal_to_process(pid, signal, false)?;

    signal::kill(Pid::from_raw(pid.try_into().unwrap()), signal)?;
    Ok(true)
}

/// This is a helper function to safely kill a child process.
/// Its purpose is to properly kill all processes and prevent any dangling processes.
pub fn kill_child(task_id: usize, child: &mut Child, _kill_children: bool) -> bool {
    match child.kill() {
        Err(_) => {
            debug!("Task {} has already finished by itself", task_id);
            false
        }
        _ => true,
    }
}

/// Send a signal to a unix process.
fn send_signal_to_process(pid: u32, signal: Signal, _children: bool) -> Result<bool, nix::Error> {
    debug!("Sending signal {} to {}", signal, pid);

    signal::kill(Pid::from_raw(pid.try_into().unwrap()), signal)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    /// THIS DOESN'T WORK YET
    /// psutil doesn't really have a way to check whether a process is actually gone.
    ///
    /// Assert that certain process id no longer exists
    fn process_is_gone(_pid: u32) -> bool {
        //match Process::new(pid) {
        //    Ok(process) => !process.is_running(),
        //    Err(_) => true,
        //}
        true
    }

    #[test]
    /// Simply check, whether spawning of a shell command works
    fn test_spawn_command() {
        let mut child = compile_shell_command("sleep 0.1")
            .spawn()
            .expect("Failed to spawn echo");

        let ecode = child.wait().expect("failed to wait on echo");

        assert!(ecode.success());
    }

    #[test]
    /// Ensure a `sh -c` command will be properly killed without detached processes.
    fn test_shell_command_is_killed() {
        let mut child = compile_shell_command("sleep 60 & sleep 60 && echo 'this is a test'")
            .spawn()
            .expect("Failed to spawn echo");
        let pid = child.id();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, false));

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid));
    }

    #[test]
    /// Ensure a normal command without `sh -c` will be killed.
    fn test_normal_command_is_killed() {
        let mut child = Command::new("sleep")
            .arg("60")
            .spawn()
            .expect("Failed to spawn echo");
        let pid = child.id();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, false));

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        assert!(process_is_gone(pid));
    }
}
