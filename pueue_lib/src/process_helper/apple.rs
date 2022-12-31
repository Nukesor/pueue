use libproc::libproc::{proc_pid, task_info};

/// Check, whether a specific process exists or not
pub fn process_exists(pid: u32) -> bool {
    proc_pid::pidinfo::<task_info::TaskInfo>(pid.try_into().unwrap(), 0).is_ok()
}

#[cfg(test)]
pub mod tests {
    // TODO: swap darwin_libproc out for libproc when that project supports listing pids by
    // group id.
    use darwin_libproc;
    use log::warn;

    pub fn get_process_group_pids(pgrp: i32) -> Vec<i32> {
        match darwin_libproc::pgrp_only_pids(pgrp) {
            Err(error) => {
                warn!("Failed to get list of processes in process group {pgrp}: {error}");
                Vec::new()
            }
            Ok(processes) => processes,
        }
    }
}
