use libproc::libproc::{proc_pid, task_info};

/// Check, whether a specific process exists or not
pub fn process_exists(pid: u32) -> bool {
    proc_pid::pidinfo::<task_info::TaskInfo>(pid.try_into().unwrap(), 0).is_ok()
}
