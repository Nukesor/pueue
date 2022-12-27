use std::collections::HashMap;
use std::thread;

use anyhow::Result;
use tokio::time::sleep;

use crate::fixtures::*;
use crate::helper::*;

/// Test that `wait` will detect new commands and wait until all queued commands are done.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn multiple_tasks() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Run a command that'll run for a short time after a delay.
    // The `pueue wait` command will be spawne directly afterwards, resulting in the spawned
    // process to wait for this command to finish.
    run_client_command(shared, &["add", "--delay", "2 seconds", "sleep 1"])?;

    // Spawn the `pueue wait` command in a separate thread.
    // We expect it to finish later on its own.
    let shared_clone = shared.clone();
    let wait_handle = thread::spawn(move || run_client_command(&shared_clone, &["wait"]));
    // Sleep for half a second to give `pueue wait` time to properly start.
    sleep(std::time::Duration::from_millis(500)).await;

    // We now spawn another task that should be picked up by and waited upon completion
    // by the `wait` process.
    run_client_command(shared, &["add", "--after=0", "sleep 1"])?;

    let output = wait_handle.join().unwrap()?;
    let log = String::from_utf8_lossy(&output.stdout);
    // All lines have the following pattern:
    // 01:49:42 - New task 1 with status Queued
    //
    // The following code trims all timestamps from the log output.
    // We cannot work with proper timings, as these times are determined by the client.
    // They are unknown to us.
    let mut log = log
        .lines()
        .map(|line| line.split('-').nth(1).unwrap().trim_start())
        .collect::<Vec<&str>>()
        .join("\n");
    log.push('\n');

    assert_stdout_matches(
        "wait__multiple_tasks",
        log.as_bytes().to_owned(),
        HashMap::new(),
    )?;

    Ok(())
}
