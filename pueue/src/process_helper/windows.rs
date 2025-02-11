// We allow color_eyre in here, as this is a module that'll be strictly used internally.
// As soon as it's obvious that this is code is intended to be exposed to library users, we have to
// go ahead and replace any `anyhow` usage by proper error handling via our own Error type.
use command_group::GroupChild;
use pueue_lib::Settings;
use winapi::{
    shared::{minwindef::FALSE, ntdef::NULL},
    um::{
        errhandlingapi::GetLastError,
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        processthreadsapi::{OpenThread, ResumeThread, SuspendThread},
        tlhelp32::{
            CreateToolhelp32Snapshot, Process32First, Process32Next, Thread32First, Thread32Next,
            PROCESSENTRY32, TH32CS_SNAPPROCESS, TH32CS_SNAPTHREAD, THREADENTRY32,
        },
        winnt::THREAD_SUSPEND_RESUME,
    },
};

use crate::internal_prelude::*;

/// Shim signal enum for windows.
pub enum Signal {
    SIGINT,
    SIGKILL,
    SIGTERM,
    SIGCONT,
    SIGSTOP,
}

pub fn get_shell_command(settings: &Settings) -> Vec<String> {
    let Some(ref shell_command) = settings.daemon.shell_command else {
        // Chain two `powershell` commands, one that sets the output encoding to utf8 and then the
        // user provided one.
        return vec![
            "powershell".into(),
            "-c".into(),
            "[Console]::OutputEncoding = [Text.UTF8Encoding]::UTF8; {{ pueue_command_string }}"
                .into(),
        ];
    };

    shell_command.clone()
}

/// Send a signal to a windows process.
pub fn send_signal_to_child(child: &mut GroupChild, signal: Signal) -> Result<()> {
    let pids = get_cur_task_processes(child.id());
    if pids.is_empty() {
        bail!("Process has just gone away");
    }

    match signal {
        Signal::SIGSTOP => {
            for pid in pids {
                for thread in get_threads(pid) {
                    suspend_thread(thread);
                }
            }
        }
        Signal::SIGCONT => {
            for pid in pids {
                for thread in get_threads(pid) {
                    resume_thread(thread);
                }
            }
        }
        _ => {
            bail!("Trying to send unix signal on a windows machine. This isn't supported.");
        }
    }

    Ok(())
}

/// Kill a child process
pub fn kill_child(task_id: usize, child: &mut GroupChild) -> std::io::Result<()> {
    match child.kill() {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::InvalidData => {
            // Process already exited
            info!("Task {task_id} has already finished by itself.");
            Ok(())
        }
        Err(err) => Err(err),
    }
}

/// Get current task pid, all child pid and all children's children
/// on the job object created by command_group.
fn get_cur_task_processes(task_pid: u32) -> Vec<u32> {
    let mut all_pids = Vec::new();

    // Get all pids by BFS
    let mut parent_pids = vec![task_pid];
    while let Some(pid) = parent_pids.pop() {
        all_pids.push(pid);

        get_child_pids(pid, &mut parent_pids);
    }

    // Keep parent pid ahead of child. We need execute action for parent process first.
    all_pids.reverse();
    all_pids
}

/// Get child pids of a specific process.
fn get_child_pids(target_pid: u32, pid_list: &mut Vec<u32>) {
    unsafe {
        // Take a snapshot of all processes in the system.
        // While enumerating the set of processes, new processes can be created and destroyed.
        let snapshot_handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, target_pid);
        if snapshot_handle == INVALID_HANDLE_VALUE {
            error!("Failed to get process {target_pid} snapShot");
            return;
        }

        // Walk the list of processes.
        let mut process_entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };
        if Process32First(snapshot_handle, &mut process_entry) == FALSE {
            error!("Couldn't get first process.");
            CloseHandle(snapshot_handle);
            return;
        }

        loop {
            if process_entry.th32ParentProcessID == target_pid {
                pid_list.push(process_entry.th32ProcessID);
            }

            if Process32Next(snapshot_handle, &mut process_entry) == FALSE {
                break;
            }
        }

        CloseHandle(snapshot_handle);
    }
}

/// Get all thread id of a specific process
fn get_threads(target_pid: u32) -> Vec<u32> {
    let mut threads = Vec::new();

    unsafe {
        // Take a snapshot of all threads in the system.
        // While enumerating the set of threads, new threads can be created and destroyed.
        let snapshot_handle = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot_handle == INVALID_HANDLE_VALUE {
            error!("Failed to get process {target_pid} snapShot");
            return threads;
        }

        // Walk the list of threads.
        let mut thread_entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        if Thread32First(snapshot_handle, &mut thread_entry) == FALSE {
            error!("Couldn't get first thread.");
            CloseHandle(snapshot_handle);
            return threads;
        }

        loop {
            if thread_entry.th32OwnerProcessID == target_pid {
                threads.push(thread_entry.th32ThreadID);
            }

            if Thread32Next(snapshot_handle, &mut thread_entry) == FALSE {
                break;
            }
        }

        CloseHandle(snapshot_handle);
    }

    threads
}

/// Suspend a thread
/// Each thread has a suspend count (with a maximum value of `MAXIMUM_SUSPEND_COUNT`).
/// If the suspend count is greater than zero, the thread is suspended; otherwise, the thread is not
/// suspended and is eligible for execution. Calling `SuspendThread` causes the target thread's
/// suspend count to be incremented. Attempting to increment past the maximum suspend count causes
/// an error without incrementing the count. [SuspendThread](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-suspendthread)
fn suspend_thread(tid: u32) {
    unsafe {
        // Attempt to convert the thread ID into a handle
        let thread_handle = OpenThread(THREAD_SUSPEND_RESUME, FALSE, tid);
        if thread_handle != NULL {
            // If SuspendThread fails, the return value is (DWORD) -1
            if u32::MAX == SuspendThread(thread_handle) {
                let err_code = GetLastError();
                warn!("Failed to suspend thread {tid} with error code {err_code}");
            }
        }

        CloseHandle(thread_handle);
    }
}

/// Resume a thread
/// ResumeThread checks the suspend count of the subject thread.
/// If the suspend count is zero, the thread is not currently suspended. Otherwise, the subject
/// thread's suspend count is decremented. If the resulting value is zero, then the execution of the
/// subject thread is resumed. [ResumeThread](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-resumethread)
fn resume_thread(tid: u32) {
    unsafe {
        // Attempt to convert the thread ID into a handle
        let thread_handle = OpenThread(THREAD_SUSPEND_RESUME, FALSE, tid);
        if thread_handle != NULL {
            // If ResumeThread fails, the return value is (DWORD) -1
            if u32::MAX == ResumeThread(thread_handle) {
                let err_code = GetLastError();
                warn!("Failed to resume thread {tid} with error code {err_code}");
            }
        }

        CloseHandle(thread_handle);
    }
}

/// Assert that certain process id no longer exists
pub fn process_exists(pid: u32) -> bool {
    unsafe {
        let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        let mut process_entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };

        loop {
            if process_entry.th32ProcessID == pid {
                CloseHandle(handle);
                return true;
            }

            if Process32Next(handle, &mut process_entry) == FALSE {
                break;
            }
        }

        CloseHandle(handle);
    }

    false
}

#[cfg(test)]
mod test {
    use std::{process::Command, thread::sleep, time::Duration};

    use command_group::CommandGroup;

    use super::*;
    use crate::process_helper::compile_shell_command;

    /// Assert that certain process id no longer exists
    fn process_is_gone(pid: u32) -> bool {
        !process_exists(pid)
    }

    /// A test helper function, which ensures that a specific amount of subprocesses can be
    /// observed for a given PID in a given time window.
    /// If the correct amount can be observed, the process ids are then returned.
    ///
    /// The process count is checked every few milliseconds for the given duration.
    fn assert_process_ids(pid: u32, expected_processes: usize, millis: usize) -> Result<Vec<u32>> {
        // Check every 50 milliseconds.
        let interval = 50;
        let tries = millis / interval;
        let mut current_try = 0;

        while current_try <= tries {
            // Continue waiting if the count doesn't match.
            let process_ids = get_cur_task_processes(pid);
            if process_ids.len() != expected_processes {
                current_try += 1;
                sleep(Duration::from_millis(interval as u64));
                continue;
            }

            return Ok(process_ids);
        }

        let count = get_cur_task_processes(pid).len();
        bail!("{expected_processes} processes were expected. Last process count was {count}")
    }

    #[test]
    fn test_spawn_command() {
        let settings = Settings::default();
        let mut child = compile_shell_command(&settings, "sleep 0.1")
            .group_spawn()
            .expect("Failed to spawn echo");

        let ecode = child.wait().expect("failed to wait on echo");

        assert!(ecode.success());
    }

    #[ignore]
    #[test]
    /// Ensure a `powershell -c` command will be properly killed without detached processes.
    ///
    /// This test is ignored for now, as it is flaky from time to time.
    /// See https://github.com/Nukesor/pueue/issues/315
    fn test_shell_command_is_killed() -> Result<()> {
        let settings = Settings::default();
        let mut child =
            compile_shell_command(&settings, "sleep 60; sleep 60; echo 'this is a test'")
                .group_spawn()
                .expect("Failed to spawn echo");
        let pid = child.id();

        // Get all processes, so we can make sure they no longer exist afterwards.
        let process_ids = assert_process_ids(pid, 1, 5000)?;

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child).is_ok());

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed.
        for pid in process_ids {
            assert!(process_is_gone(pid));
        }

        Ok(())
    }

    #[ignore]
    #[test]
    /// Ensure that a `powershell -c` process with a child process that has children of it's own
    /// will properly kill all processes and their children's children without detached processes.
    fn test_shell_command_children_are_killed() -> Result<()> {
        let settings = Settings::default();
        let mut child =
            compile_shell_command(&settings, "powershell -c 'sleep 60; sleep 60'; sleep 60")
                .group_spawn()
                .expect("Failed to spawn echo");
        let pid = child.id();
        // Get all processes, so we can make sure they no longer exist afterwards.
        let process_ids = assert_process_ids(pid, 2, 5000)?;

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child).is_ok());

        // Assert that the direct child (powershell -c) has been killed.
        sleep(Duration::from_millis(500));
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed.
        for pid in process_ids {
            assert!(process_is_gone(pid));
        }

        Ok(())
    }

    #[ignore]
    #[test]
    /// Ensure a normal command without `powershell -c` will be killed.
    fn test_normal_command_is_killed() -> Result<()> {
        let mut child = Command::new("ping")
            .arg("localhost")
            .arg("-t")
            .group_spawn()
            .expect("Failed to spawn ping");
        let pid = child.id();

        // Get all processes, so we can make sure they no longer exist afterwards.
        let _ = assert_process_ids(pid, 1, 5000)?;

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child).is_ok());

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        assert!(process_is_gone(pid));

        Ok(())
    }

    #[ignore]
    #[test]
    /// Ensure a normal command and all it's children will be
    /// properly killed without any detached processes.
    fn test_normal_command_children_are_killed() -> Result<()> {
        let mut child = Command::new("powershell")
            .arg("-c")
            .arg("sleep 60; sleep 60; sleep 60")
            .group_spawn()
            .expect("Failed to spawn echo");
        let pid = child.id();

        // Get all processes, so we can make sure they no longer exist afterwards.
        let process_ids = assert_process_ids(pid, 1, 5000)?;

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child).is_ok());

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed.
        for pid in process_ids {
            assert!(process_is_gone(pid));
        }

        Ok(())
    }
}
