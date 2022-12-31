use procfs::process;

/// Check, whether a specific process is exists or not
pub fn process_exists(pid: u32) -> bool {
    match pid.try_into() {
        Err(_) => false,
        Ok(pid) => match process::Process::new(pid) {
            Ok(process) => process.is_alive(),
            Err(_) => false,
        },
    }
}

#[cfg(test)]
pub mod tests {
    use log::warn;

    use super::process;

    /// Get all processes in a process group
    pub fn get_process_group_pids(pgrp: i32) -> Vec<i32> {
        let all_processes = match process::all_processes() {
            Err(error) => {
                warn!("Failed to get full process list: {error}");
                return Vec::new();
            }
            Ok(processes) => processes,
        };

        // Get all processes whose `stat` can be accessed without any errors.
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
}
