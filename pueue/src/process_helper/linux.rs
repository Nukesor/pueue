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
