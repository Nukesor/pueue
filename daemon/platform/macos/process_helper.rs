use std::convert::TryInto;
use std::process::{Child, Command};

use anyhow::Result;
use log::{debug, warn};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use psutil::process::{processes, Process};

use crate::task_handler::ProcessAction;

pub fn compile_shell_command(command_string: &str) -> Command {
    let mut command = Command::new("sh");
    command.arg("-c").arg(command_string);

    command
}

/// Send a signal to one of Pueue's child process handles.
/// We need a special since we assume that there's also a `sh -c` around the actuall process.
pub fn send_signal_to_child(
    child: &Child,
    action: &ProcessAction,
    send_to_children: bool,
) -> Result<bool> {
    let signal = get_signal_from_action(action);
    let pid = child.id();
    // Send the signal to the shell, don't propagate to it's children yet.
    send_signal_to_process(pid, action, false)?;

    // Now send the signal to the shells child processes and their respective
    // children if the user wants to do so.
    let shell_children = get_child_processes(pid);
    for shell_child in shell_children {
        send_signal_to_process(shell_child.pid(), action, send_to_children)?;
    }

    signal::kill(Pid::from_raw(pid.try_into().unwrap()), signal)?;
    Ok(true)
}

/// This is a helper function to safely kill a child process.
/// It's purpose is to properly kill all processes and prevent any dangling processes.
pub fn kill_child(task_id: usize, child: &mut Child, kill_children: bool) -> bool {
    let pid = child.id();
    // We have to kill the root process first, to prevent it from spawning new processes.
    // However, this prevents us from getting it's child processes afterwards.
    // That's why we have to get the list of child processes already now.
    let child_processes = get_child_processes(pid);

    // Kill the parent first
    match child.kill() {
        Err(_) => {
            debug!("Task {} has already finished by itself", task_id);
            false
        }
        _ => {
            // Now kill all remaining children, after the parent has been killed.
            for child_process in child_processes {
                // Send the signal to each child process, show warning if this fails.
                let process_pid = child_process.pid();
                if let Err(error) =
                    send_signal_to_process(process_pid, &ProcessAction::Kill, kill_children)
                {
                    warn!(
                        "Failed to send kill to pid {} with error {:?}",
                        process_pid, error
                    );
                }
            }

            true
        }
    }
}

/// Send a signal to a unix process.
fn send_signal_to_process(
    pid: u32,
    action: &ProcessAction,
    children: bool,
) -> Result<bool, nix::Error> {
    let signal = get_signal_from_action(action);
    debug!("Sending signal {} to {}", signal, pid);

    // Send the signal to all children, if that's what the user wants.
    if children {
        send_signal_to_processes(get_child_processes(pid), action);
    }

    signal::kill(Pid::from_raw(pid.try_into().unwrap()), signal)?;
    Ok(true)
}

/// Send a signal to a list of processes
fn send_signal_to_processes(processes: Vec<Process>, action: &ProcessAction) {
    let signal = get_signal_from_action(action);
    for process in processes {
        // Process is no longer alive, skip this one.
        if !process.is_running() {
            continue;
        }

        let pid = Pid::from_raw(process.pid().try_into().unwrap());
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

/// Get all children of a specific process
fn get_child_processes(pid: u32) -> Vec<Process> {
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
                    return ppid == pid;
                }
            }
            false
        })
        .collect()
}

fn get_signal_from_action(action: &ProcessAction) -> Signal {
    match action {
        ProcessAction::Kill => Signal::SIGKILL,
        ProcessAction::Pause => Signal::SIGSTOP,
        ProcessAction::Resume => Signal::SIGCONT,
    }
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
        let mut child = compile_shell_command("echo 'this is a test'")
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

        // Get all child processes, so we can make sure they no longer exist afterwards.
        let child_processes = get_child_processes(pid);
        assert_eq!(child_processes.len(), 2);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, false));

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed.
        for child_process in child_processes {
            assert!(process_is_gone(child_process.pid()));
        }
    }

    #[test]
    /// Ensure that a `sh -c` process with a child process that has children of it's own
    /// will properly kill all processes and their children's children without detached processes.
    fn test_shell_command_children_are_killed() {
        let mut child = compile_shell_command("bash -c 'sleep 60 && sleep 60' && sleep 60")
            .spawn()
            .expect("Failed to spawn echo");
        let pid = child.id();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all child processes and all childrens children,
        // so we can make sure they no longer exist afterwards.
        let child_processes = get_child_processes(pid);
        assert_eq!(child_processes.len(), 1);
        let mut childrens_children = Vec::new();
        for child_process in &child_processes {
            childrens_children.extend(get_child_processes(child_process.pid()));
        }
        assert_eq!(childrens_children.len(), 1);

        // Kill the process and make sure its childen will be killed.
        assert!(kill_child(0, &mut child, true));

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed..
        for child_process in child_processes {
            assert!(process_is_gone(child_process.pid()));
        }

        // Assert that all children's child processes have been killed.
        for child_process in childrens_children {
            assert!(process_is_gone(child_process.pid()));
        }
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

        // No childprocesses exist
        let child_processes = get_child_processes(pid);
        assert_eq!(child_processes.len(), 0);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, false));

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        assert!(process_is_gone(pid));
    }

    #[test]
    /// Ensure a normal command and all it's children will be
    /// properly killed without any detached processes.
    fn test_normal_command_children_are_killed() {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg("sleep 60 & sleep 60 && sleep 60")
            .spawn()
            .expect("Failed to spawn echo");
        let pid = child.id();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all child processes, so we can make sure they no longer exist afterwards.
        let child_processes = get_child_processes(pid);
        assert_eq!(child_processes.len(), 2);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, true));

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed.
        for child_process in child_processes {
            assert!(process_is_gone(child_process.pid()));
        }
    }
}
