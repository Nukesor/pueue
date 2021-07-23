use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Child;

use anyhow::{anyhow, bail, Context, Result};
use procfs::process::Process;
use pueue_lib::network::message::Message;
use pueue_lib::settings::Shared;
use pueue_lib::state::GroupStatus;

use super::{get_state, sleep_ms};

pub fn assert_success(message: Message) {
    assert!(matches!(message, Message::Success(_)));
}

pub fn assert_failure(message: Message) {
    assert!(matches!(message, Message::Failure(_)));
}

/// Get a daemon pid from a specific pueue directory.
/// This function gives the daemon a little time to boot up, but ultimately crashes if it takes too
/// long.
pub fn get_pid(pueue_dir: &Path) -> Result<i32> {
    let pid_file = pueue_dir.join("pueue.pid");

    // Give the daemon about 1 sec to boot and create the pid file.
    let tries = 20;
    let mut current_try = 0;

    while current_try < tries {
        // The daemon didn't create the pid file yet. Wait for 100ms and try again.
        if !pid_file.exists() {
            sleep_ms(50);
            current_try += 1;
            continue;
        }

        let mut file = File::open(&pid_file).context("Couldn't open pid file")?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .context("Couldn't write to file")?;

        // The file has been created but not yet been written to.
        if content.is_empty() {
            sleep_ms(50);
            current_try += 1;
            continue;
        }

        let pid = content
            .parse::<i32>()
            .map_err(|_| anyhow!("Couldn't parse value: {}", content))?;
        return Ok(pid);
    }

    bail!("Couldn't find pid file after about 1 sec.");
}

/// Waits for a daemon to shut down.
/// This is done by waiting for the pid to disappear.
pub fn wait_for_shutdown(pid: i32) -> Result<()> {
    // Try to read the process. If this fails, the daemon already exited.
    let process = match Process::new(pid) {
        Ok(process) => process,
        Err(_) => return Ok(()),
    };

    // Give the daemon about 1 sec to shutdown.
    let tries = 40;
    let mut current_try = 0;

    while current_try < tries {
        // Process is still alive, wait a little longer
        if process.is_alive() {
            sleep_ms(50);
            current_try += 1;
            continue;
        }

        return Ok(());
    }

    bail!("Couldn't find pid file after about 2 sec.");
}

/// Waits for a status on a specific group.
pub async fn wait_for_group_status(
    shared: &Shared,
    group: &str,
    _expected_status: GroupStatus,
) -> Result<()> {
    let state = get_state(shared).await?;

    // Give the daemon about 1 sec to shutdown.
    let tries = 20;
    let mut current_try = 0;

    while current_try < tries {
        // Process is still alive, wait a little longer
        if let Some(status) = state.groups.get(group) {
            if matches!(status, _expected_status) {
                return Ok(());
            }
        }

        sleep_ms(50);
        current_try += 1;
    }

    bail!(
        "Group {} didn't change to state {:?} after about 1 sec.",
        group,
        _expected_status
    );
}

pub fn kill_and_print_output(mut child: Child) -> Result<()> {
    let _ = child.kill();
    let output = child.wait_with_output()?;
    println!("Stdout: \n{:?}", String::from_utf8_lossy(&output.stdout));

    println!("Stderr: \n{:?}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}
