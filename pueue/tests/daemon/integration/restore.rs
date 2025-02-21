use pueue_lib::{GroupStatus, network::message::TaskSelection};
use rstest::rstest;

use crate::{helper::*, internal_prelude::*};

/// The daemon should start in the same state as before shutdown, if no tasks are queued.
/// This function tests for the running state.
#[rstest]
#[case(true)]
#[case(false)]
#[tokio::test]
async fn test_start_running(#[case] compress: bool) -> Result<()> {
    let (mut settings, tempdir) = daemon_base_setup()?;
    settings.daemon.compress_state_file = compress;
    settings
        .save(&Some(tempdir.path().join("pueue.yml")))
        .context("Couldn't write pueue config to temporary directory")?;

    let mut child = standalone_daemon(&settings.shared).await?;
    let shared = &settings.shared;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(shared).await?);
    wait_for_shutdown(&mut child).await?;

    // Boot it up again
    let mut child = standalone_daemon(&settings.shared).await?;

    assert_group_status(
        shared,
        PUEUE_DEFAULT_GROUP,
        GroupStatus::Running,
        "Default group should still be running.",
    )
    .await?;

    child.kill()?;
    Ok(())
}

/// The daemon should start in the same state as before shutdown, if no tasks are queued.
/// This function tests for the paused state.
#[rstest]
#[case(true)]
#[case(false)]
#[tokio::test]
async fn test_start_paused(#[case] compress: bool) -> Result<()> {
    let (mut settings, tempdir) = daemon_base_setup()?;
    settings.daemon.compress_state_file = compress;
    settings
        .save(&Some(tempdir.path().join("pueue.yml")))
        .context("Couldn't write pueue config to temporary directory")?;

    let mut child = standalone_daemon(&settings.shared).await?;
    let shared = &settings.shared;

    // This pauses the daemon
    pause_tasks(shared, TaskSelection::All).await?;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(shared).await?);
    wait_for_shutdown(&mut child).await?;

    // Boot it up again
    let mut child = standalone_daemon(&settings.shared).await?;

    assert_group_status(
        shared,
        PUEUE_DEFAULT_GROUP,
        GroupStatus::Paused,
        "Default group should still be paused.",
    )
    .await?;

    child.kill()?;
    Ok(())
}
