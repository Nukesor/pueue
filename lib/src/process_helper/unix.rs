use std::process::Command;

// We allow anyhow in here, as this is a module that'll be strictly used internally.
// As soon as it's obvious that this is code is intended to be exposed to library users, we have to
// go ahead and replace any `anyhow` usage by proper error handling via our own Error type.
use anyhow::Result;
use command_group::{GroupChild, Signal, UnixChildExt};
use log::info;

pub fn compile_shell_command(command_string: &str) -> Command {
    let mut command = Command::new("sh");
    command.arg("-c").arg(command_string);

    command
}

/// Send a signal to one of Pueue's child process or process group handles.
pub fn send_signal_to_child<T>(
    child: &mut GroupChild,
    signal: T,
    send_to_children: bool,
) -> Result<()>
where
    T: Into<Signal>,
{
    if send_to_children {
        // Send the signal to the process group
        child.signal(signal.into())?;
    } else {
        // Send the signal to the process itself
        child.inner().signal(signal.into())?;
    }
    Ok(())
}

/// This is a helper function to safely kill a child process or process group.
/// Its purpose is to properly kill all processes and prevent any dangling processes.
pub fn kill_child(
    task_id: usize,
    child: &mut GroupChild,
    kill_children: bool,
) -> std::io::Result<()> {
    let result = if kill_children {
        child.kill()
    } else {
        child.inner().kill()
    };
    match result {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::InvalidData => {
            // Process already exited
            info!("Task {task_id} has already finished by itself.");
            Ok(())
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;

    use anyhow::Result;
    use command_group::CommandGroup;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::process_helper::process_exists;
    use crate::process_helper::tests::get_process_group_pids;

    /// Assert that certain process id no longer exists
    fn process_is_gone(pid: u32) -> bool {
        !process_exists(pid)
    }

    #[test]
    fn test_spawn_command() {
        let mut child = compile_shell_command("sleep 0.1")
            .group_spawn()
            .expect("Failed to spawn echo");

        let ecode = child.wait().expect("failed to wait on echo");

        assert!(ecode.success());
    }

    #[test]
    /// Ensure a `sh -c` command will be properly killed without detached processes.
    fn test_shell_command_is_killed() -> Result<()> {
        let mut child = compile_shell_command("sleep 60 & sleep 60 && echo 'this is a test'")
            .group_spawn()
            .expect("Failed to spawn echo");
        let pid: i32 = child.id().try_into().unwrap();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all child processes, so we can make sure they no longer exist afterwards.
        // The process group id is the same as the parent process id.
        let group_pids = get_process_group_pids(pid);
        assert_eq!(group_pids.len(), 3);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, true).is_ok());

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));
        // collect the exit status; otherwise the child process hangs around as a zombie.
        child.try_wait().unwrap_or_default();

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid as u32));

        // Assert that all child processes have been killed.
        assert_eq!(get_process_group_pids(pid).len(), 0);

        Ok(())
    }

    #[test]
    /// Ensure a `sh -c` command will be properly killed without detached processes when using unix
    /// signals directly.
    fn test_shell_command_is_killed_with_signal() -> Result<()> {
        let mut child = compile_shell_command("sleep 60 & sleep 60 && echo 'this is a test'")
            .group_spawn()
            .expect("Failed to spawn echo");
        let pid: i32 = child.id().try_into().unwrap();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all child processes, so we can make sure they no longer exist afterwards.
        // The process group id is the same as the parent process id.
        let group_pids = get_process_group_pids(pid);
        assert_eq!(group_pids.len(), 3);

        // Kill the process and make sure it'll be killed.
        send_signal_to_child(&mut child, Signal::SIGKILL, true).unwrap();

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));
        // collect the exit status; otherwise the child process hangs around as a zombie.
        child.try_wait().unwrap_or_default();

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid as u32));

        // Assert that all child processes have been killed.
        assert_eq!(get_process_group_pids(pid).len(), 0);

        Ok(())
    }

    #[test]
    /// Ensure that a `sh -c` process with a child process that has children of its own
    /// will properly kill all processes and their children's children without detached processes.
    fn test_shell_command_children_are_killed() -> Result<()> {
        let mut child = compile_shell_command("bash -c 'sleep 60 && sleep 60' && sleep 60")
            .group_spawn()
            .expect("Failed to spawn echo");
        let pid: i32 = child.id().try_into().unwrap();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all child processes, so we can make sure they no longer exist afterwards.
        // The process group id is the same as the parent process id.
        let group_pids = get_process_group_pids(pid);
        assert_eq!(group_pids.len(), 3);

        // Kill the process and make sure its childen will be killed.
        assert!(kill_child(0, &mut child, true).is_ok());

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));
        // collect the exit status; otherwise the child process hangs around as a zombie.
        child.try_wait().unwrap_or_default();

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid as u32));

        // Assert that all child processes have been killed.
        assert_eq!(get_process_group_pids(pid).len(), 0);

        Ok(())
    }

    #[test]
    /// Ensure a normal command without `sh -c` will be killed.
    fn test_normal_command_is_killed() -> Result<()> {
        let mut child = Command::new("sleep")
            .arg("60")
            .group_spawn()
            .expect("Failed to spawn echo");
        let pid: i32 = child.id().try_into().unwrap();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // No further processes exist in the group
        let group_pids = get_process_group_pids(pid);
        assert_eq!(group_pids.len(), 1);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, false).is_ok());

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));
        // collect the exit status; otherwise the child process hangs around as a zombie.
        child.try_wait().unwrap_or_default();

        assert!(process_is_gone(pid as u32));

        Ok(())
    }

    #[test]
    /// Ensure a normal command and all its children will be
    /// properly killed without any detached processes.
    fn test_normal_command_children_are_killed() -> Result<()> {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg("sleep 60 & sleep 60 && sleep 60")
            .group_spawn()
            .expect("Failed to spawn echo");
        let pid: i32 = child.id().try_into().unwrap();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all child processes, so we can make sure they no longer exist afterwards.
        // The process group id is the same as the parent process id.
        let group_pids = get_process_group_pids(pid);
        assert_eq!(group_pids.len(), 3);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, true).is_ok());

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));
        // collect the exit status; otherwise the child process hangs around as a zombie.
        child.try_wait().unwrap_or_default();

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid as u32));

        // Assert that all child processes have been killed.
        assert_eq!(get_process_group_pids(pid).len(), 0);

        Ok(())
    }

    #[test]
    /// Ensure a command with children can be killed separately from
    /// the child process.
    fn test_kill_command_only() -> Result<()> {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg("sleep 60 & sleep 60 && sleep 60")
            .group_spawn()
            .expect("Failed to spawn echo");
        let pid: i32 = child.id().try_into().unwrap();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all child processes, so we can make sure they no longer exist afterwards.
        // The process group id is the same as the parent process id.
        let group_pids = get_process_group_pids(pid);
        assert_eq!(group_pids.len(), 3);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, false).is_ok());

        // Sleep a little to give the process time to shutdown.
        sleep(Duration::from_millis(500));
        // collect the exit status; otherwise the child process hangs around as a zombie.
        child.try_wait().unwrap_or_default();

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid as u32));
        assert!(kill_child(0, &mut child, false).is_err());

        // Assert that the remaining processes are still there
        assert_eq!(get_process_group_pids(pid).len(), 2);

        // Kill the group
        assert!(kill_child(0, &mut child, true).is_ok());
        sleep(Duration::from_millis(500));
        assert_eq!(get_process_group_pids(pid).len(), 0);

        Ok(())
    }
}
