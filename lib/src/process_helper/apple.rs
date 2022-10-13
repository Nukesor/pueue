use log::warn;

use darwin_libproc;
use libc;

pub fn get_process_group_pids(pgrp: libc::pid_t) -> Vec<libc::pid_t> {
    match darwin_libproc::pgrp_only_pids(pgrp) {
        Err(error) => {
            warn!("Failed to get list of processes in process group {pgrp}: {error}");
            Vec::new()
        }
        Ok(processes) => processes,
    }
}

/// Check, whether a specific process exists or not
pub fn process_exists(pid: u32) -> bool {
    darwin_libproc::task_info(pid.try_into().unwrap()).is_ok()
}
