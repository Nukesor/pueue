use log::warn;

use libc;
use procfs::process;

/// Get all processes in a process group
pub fn get_process_group_pids(pgrp: libc::pid_t) -> Vec<libc::pid_t> {
    let all_processes = match process::all_processes() {
        Err(error) => {
            warn!("Failed to get full process list: {error}");
            return Vec::new();
        }
        Ok(processes) => processes,
    };

    // Get all processes whose `stat` can be access without any errors.
    // If the stat() result matches the process group, use the process PID.
    all_processes
        .into_iter()
        .filter_map(|result| result.ok())
        .filter_map(|process| match process.stat() {
            Ok(stat) if stat.pgrp == pgrp => Some(process.pid),
            _ => None,
        })
        .collect()
}

/// Check, whether a specific process is exists or not
pub fn process_exists(pid: u32) -> bool {
    match process::Process::new(pid.try_into().unwrap()) {
        Ok(process) => process.is_alive(),
        Err(_) => false,
    }
}
