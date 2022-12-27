use anyhow::{Context, Result};

use crate::client::helper::*;

/// Test that the local `follow` command works as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it started.
    assert_success(add_task(shared, "sleep 1 && echo test", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    // Execute `follow`.
    // This will result in the client receiving the streamed output until the task finished.
    let output = run_client_command(shared, &["follow"])?;

    assert_snapshot_matches_stdout("follow__default", output.stdout)?;

    Ok(())
}

/// Test that the `follow` command works with the log being streamed by the daemon works as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote() -> Result<()> {
    let mut daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Force the client to read remote logs via config file.
    daemon.settings.client.read_local_logs = false;
    // Persist the change, so it can be seen by the client.
    daemon
        .settings
        .save(&Some(daemon.tempdir.path().join("pueue.yml")))
        .context("Couldn't write pueue config to temporary directory")?;

    // Add a task and wait until it started.
    assert_success(add_task(shared, "sleep 1 && echo test", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    // Execute `follow`.
    // This will result in the client receiving the streamed output until the task finished.
    let output = run_client_command(shared, &["follow"])?;

    assert_snapshot_matches_stdout("follow__default", output.stdout)?;

    Ok(())
}

/// Test that the remote `follow` command works, if one specifies to only show the last few lines
/// of recent output.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_with_last_lines() -> Result<()> {
    let mut daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Force the client to read remote logs via config file.
    daemon.settings.client.read_local_logs = false;
    // Persist the change, so it can be seen by the client.
    daemon
        .settings
        .save(&Some(daemon.tempdir.path().join("pueue.yml")))
        .context("Couldn't write pueue config to temporary directory")?;

    // Add a task which echos 8 lines of output
    assert_success(add_task(shared, "echo \"1\n2\n3\n4\n5\n6\n7\n8\" && sleep 1", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    // Execute `follow`.
    // This will result in the client receiving the streamed output until the task finished.
    let output = run_client_command(shared, &["follow", "--lines=4"])?;

    assert_snapshot_matches_stdout("follow__last_lines", output.stdout)?;

    Ok(())
}
