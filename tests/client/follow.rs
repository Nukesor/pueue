use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;

use crate::fixtures::*;
use crate::helper::*;

/// Test that the `follow` command works with the log being streamed by the daemon works as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_follow() -> Result<()> {
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
    let output = run_client_command(shared, &["follow"]).await?;

    assert_stdout_matches("follow__default_follow", output.stdout, HashMap::new())?;

    Ok(())
}
