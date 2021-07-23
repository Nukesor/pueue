use std::process::{Child, Command};

use anyhow::{bail, Result};
use log::{error, info, warn};
use winapi::shared::minwindef::FALSE;
use winapi::shared::ntdef::NULL;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::{
    OpenProcess, OpenThread, ResumeThread, SuspendThread, TerminateProcess,
};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, Thread32First, Thread32Next,
    PROCESSENTRY32, TH32CS_SNAPPROCESS, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use winapi::um::winnt::{PROCESS_TERMINATE, THREAD_SUSPEND_RESUME};

use crate::task_handler::ProcessAction;
use pueue_lib::network::message::Signal as InternalSignal;

pub fn compile_shell_command(command_string: &str) -> Command {
    // Chain two `powershell` commands, one that sets the output encoding to utf8 and then the user provided one.
    let mut command = Command::new("powershell");
    command.arg("-c").arg(format!(
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::UTF8; {}",
        command_string
    ));

    command
}

pub fn send_internal_signal_to_child(
    child: &Child,
    signal: InternalSignal,
    send_to_children: bool,
) -> Result<bool> {
    bail!("Trying to send unix signal on a windows machine. This isn't supported.");
}

/// Send a signal to a windows process.
pub fn run_action_on_child(child: &Child, action: &ProcessAction, _children: bool) -> Result<bool> {
    let pids = get_cur_task_processes(child.id());
    if pids.is_empty() {
        bail!("Process has just gone away");
    }

    match action {
        ProcessAction::Pause => {
            for pid in pids {
                for thread in get_threads(pid) {
                    suspend_thread(thread);
                }
            }
        }
        ProcessAction::Resume => {
            for pid in pids {
                for thread in get_threads(pid) {
                    resume_thread(thread);
                }
            }
        }
    }

    Ok(true)
}

/// Kill a child process
pub fn kill_child(task_id: usize, child: &mut Child, _kill_children: bool) -> bool {
    match child.kill() {
        Err(_) => {
            info!("Task {} has already finished by itself", task_id);
            false
        }
        Ok(_) => {
            let pids = get_cur_task_processes(child.id());

            for pid in pids {
                terminate_process(pid);
            }
            true
        }
    }
}

/// Get current task pid, all child pid and all children's children
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
            error!("Failed to get process {} snapShot", target_pid);
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
            error!("Failed to get process {} snapShot", target_pid);
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
/// If the suspend count is greater than zero, the thread is suspended; otherwise, the thread is not suspended and is eligible for execution.
/// Calling `SuspendThread` causes the target thread's suspend count to be incremented.
/// Attempting to increment past the maximum suspend count causes an error without incrementing the count.
/// [SuspendThread](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-suspendthread)
fn suspend_thread(tid: u32) {
    unsafe {
        // Attempt to convert the thread ID into a handle
        let thread_handle = OpenThread(THREAD_SUSPEND_RESUME, FALSE, tid);
        if thread_handle != NULL {
            // If SuspendThread fails, the return value is (DWORD) -1
            if u32::max_value() == SuspendThread(thread_handle) {
                let err_code = GetLastError();
                warn!(
                    "Failed to suspend thread {} with error code {}",
                    tid, err_code
                );
            }
        }

        CloseHandle(thread_handle);
    }
}

/// Resume a thread
/// ResumeThread checks the suspend count of the subject thread.
/// If the suspend count is zero, the thread is not currently suspended. Otherwise, the subject thread's suspend count is decremented.
/// If the resulting value is zero, then the execution of the subject thread is resumed.
/// [ResumeThread](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-resumethread)
fn resume_thread(tid: u32) {
    unsafe {
        // Attempt to convert the thread ID into a handle
        let thread_handle = OpenThread(THREAD_SUSPEND_RESUME, FALSE, tid);
        if thread_handle != NULL {
            // If ResumeThread fails, the return value is (DWORD) -1
            if u32::max_value() == ResumeThread(thread_handle) {
                let err_code = GetLastError();
                warn!(
                    "Failed to resume thread {} with error code {}",
                    tid, err_code
                );
            }
        }

        CloseHandle(thread_handle);
    }
}

/// Terminate a process
/// [TerminateProcess](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess)
fn terminate_process(pid: u32) {
    unsafe {
        // Get a handle for the target process
        let process_handle = OpenProcess(PROCESS_TERMINATE, FALSE, pid);
        // If TerminateProcess fails, the return value is zero.
        if 0 == TerminateProcess(process_handle, 1) {
            let err_code = GetLastError();
            warn!(
                "Failed to terminate process {} with error code {}",
                pid, err_code
            );
        }

        CloseHandle(process_handle);
    }
}

/// Assert that certain process id no longer exists
pub fn process_exists(pid: u32) -> bool {
    unsafe {
        let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        let mut process_entry = PROCESSENTRY32::default();
        process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

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
    use std::thread::sleep;
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::*;

    /// Assert that certain process id no longer exists
    fn process_is_gone(pid: u32) -> bool {
        !process_exists(pid)
    }

    #[test]
    fn test_spawn_command() {
        let mut child = compile_shell_command("sleep 0.1")
            .spawn()
            .expect("Failed to spawn echo");

        let ecode = child.wait().expect("failed to wait on echo");

        assert!(ecode.success());
    }

    #[test]
    /// Ensure a `powershell -c` command will be properly killed without detached processes.
    fn test_shell_command_is_killed() {
        let mut child = compile_shell_command("sleep 60; sleep 60; echo 'this is a test'")
            .spawn()
            .expect("Failed to spawn echo");
        let pid = child.id();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(1000));

        // Get all processes, so we can make sure they no longer exist afterwards.
        let process_ids = get_cur_task_processes(pid);
        assert_eq!(process_ids.len(), 1);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, false));

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed.
        for pid in process_ids {
            assert!(process_is_gone(pid));
        }
    }

    #[test]
    /// Ensure that a `powershell -c` process with a child process that has children of it's own
    /// will properly kill all processes and their children's children without detached processes.
    fn test_shell_command_children_are_killed() {
        let mut child = compile_shell_command("powershell -c 'sleep 60; sleep 60'; sleep 60")
            .spawn()
            .expect("Failed to spawn echo");
        let pid = child.id();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(1000));

        let process_ids = get_cur_task_processes(pid);
        assert_eq!(process_ids.len(), 2);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, false));

        // Assert that the direct child (powershell -c) has been killed.
        sleep(Duration::from_millis(500));
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed.
        for pid in process_ids {
            assert!(process_is_gone(pid));
        }
    }

    #[test]
    /// Ensure a normal command without `powershell -c` will be killed.
    fn test_normal_command_is_killed() {
        let mut child = Command::new("ping")
            .arg("localhost")
            .arg("-t")
            .spawn()
            .expect("Failed to spawn ping");
        let pid = child.id();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all processes
        let process_ids = get_cur_task_processes(pid);
        assert_eq!(process_ids.len(), 1);

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
        let mut child = Command::new("powershell")
            .arg("-c")
            .arg("sleep 60; sleep 60; sleep 60")
            .spawn()
            .expect("Failed to spawn echo");
        let pid = child.id();
        // Sleep a little to give everything a chance to spawn.
        sleep(Duration::from_millis(500));

        // Get all processes, so we can make sure they no longer exist afterwards.
        let process_ids = get_cur_task_processes(pid);
        assert_eq!(process_ids.len(), 1);

        // Kill the process and make sure it'll be killed.
        assert!(kill_child(0, &mut child, true));

        // Sleep a little to give all processes time to shutdown.
        sleep(Duration::from_millis(500));

        // Assert that the direct child (sh -c) has been killed.
        assert!(process_is_gone(pid));

        // Assert that all child processes have been killed.
        for pid in process_ids {
            assert!(process_is_gone(pid));
        }
    }
}
