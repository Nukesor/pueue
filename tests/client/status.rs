use anyhow::Context;
use anyhow::Result;
use pueue_lib::state::State;
use rstest::rstest;

use crate::fixtures::*;
use crate::helper::*;

/// Test that the normal status command works as expected.
/// Calling `pueue` without any subcommand is equivalent of using `status`.
#[rstest]
#[case(false)]
#[case(true)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default(#[case] use_subcommand: bool) -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    assert_success(add_task(shared, "ls", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let subcommand = if use_subcommand {
        vec!["status"]
    } else {
        Vec::new()
    };

    let output = run_client_command(shared, &subcommand)?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("status__default", output.stdout, context)?;

    Ok(())
}

/// Test the status output with all columns enabled.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn full() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a paused task so we can use it as a dependency.
    run_client_command(
        shared,
        &["add", "--label", "test", "--delay", "1 minute", "ls"],
    )?;

    // Add a second command that depends on the first one.
    run_client_command(shared, &["add", "--after=0", "ls"])?;

    let output = run_client_command(shared, &["status"])?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("status__full", output.stdout, context)?;

    Ok(())
}

/// Calling `status` with the `--color=always` flag, colors the output as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn colored() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    assert_success(add_task(shared, "ls", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["--color", "always", "status"])?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("status__colored", output.stdout, context)?;

    Ok(())
}

/// Calling `pueue status --json` will result in the current state being printed to the cli.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn json() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    assert_success(add_task(shared, "ls", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["status", "--json"])?;

    let json = String::from_utf8_lossy(&output.stdout);
    let deserialized_state: State =
        serde_json::from_str(&*json).context("Failed to deserialize json state")?;

    let state = get_state(shared).await?;
    assert_eq!(
        deserialized_state, *state,
        "Json state differs from actual daemon state."
    );

    Ok(())
}
