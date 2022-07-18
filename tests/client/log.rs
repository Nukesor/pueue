use std::collections::BTreeMap;
use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use pueue_lib::task::Task;
use serde_derive::Deserialize;

use crate::fixtures::*;
use crate::helper::*;

/// Test that the normal `log` command works as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default_log() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    assert_success(add_task(shared, "echo test", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["log"]).await?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("log__default_log", output.stdout, context)?;

    Ok(())
}

/// Test that the `log` command works with the log being streamed by the daemon works as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_log() -> Result<()> {
    let mut daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Force the client to read remote logs via config file.
    daemon.settings.client.read_local_logs = false;
    // Persist the change, so it can be seen by the client.
    daemon
        .settings
        .save(&Some(daemon.tempdir.path().join("pueue.yml")))
        .context("Couldn't write pueue config to temporary directory")?;

    // Add a task and wait until it finishes.
    assert_success(add_task(shared, "echo test", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["log"]).await?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("log__default_log", output.stdout, context)?;

    Ok(())
}

/// Calling `log` with the `--color=always` flag, colors the output as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn colored_log() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    assert_success(add_task(shared, "echo test", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["--color", "always", "log"]).await?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("log__log_with_color", output.stdout, context)?;

    Ok(())
}

/// This is the output struct used for task logs.
/// Since the Pueue client isn't exposed as a library, we have to declare our own for testing
/// purposes. The counter part can be found in `client/display/log/json.rs`.
#[derive(Debug, Deserialize)]
pub struct TaskLog {
    pub task: Task,
    pub output: String,
}

/// Calling `pueue log --json` prints the expected json output to stdout.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn status_json() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    assert_success(add_task(shared, "echo test", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["log", "--json"]).await?;

    // Deserialize the json back to the original task BTreeMap.
    let json = String::from_utf8_lossy(&output.stdout);
    let mut task_logs: BTreeMap<usize, TaskLog> = serde_json::from_str(&*json)
        .context(format!("Failed to deserialize json tasks: \n{json}"))?;

    // Get the actual BTreeMap from the daemon
    let mut state = get_state(shared).await?;
    let original_task = state.tasks.get_mut(&0).unwrap();
    // Clean the environment variables, as they aren't transmitted when calling `log`.
    original_task.envs = HashMap::new();

    let task_log = task_logs.get_mut(&0).expect("Expected one task log");
    assert_eq!(
        original_task, &task_log.task,
        "Deserialized task and original task aren't equal"
    );

    // Append a newline to the deserialized task's output, which is automatically done when working
    // with the shell.
    task_log.output.push('\n');

    assert_stdout_matches(
        "log__json_log_output",
        task_log.output.clone().into(),
        HashMap::new(),
    )?;

    Ok(())
}
