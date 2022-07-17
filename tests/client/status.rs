use anyhow::Result;
use rstest::rstest;

use crate::fixtures::*;
use crate::helper::*;

/// Test that the normal status command works as expected.
/// Calling `pueue` without any subcommand is equivalent of using `status`.
#[rstest]
#[case(false)]
#[case(true)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default_status(#[case] use_subcommand: bool) -> Result<()> {
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

    let output = run_client_command(shared, &subcommand).await?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("status__default_status", output.stdout, context)?;

    Ok(())
}

/// Calling `status` with the `--color=always` flag, colors the output as expected.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn colored_status() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    assert_success(add_task(shared, "ls", false).await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["--color", "always", "status"]).await?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("status__status_with_color", output.stdout, context)?;

    Ok(())
}
