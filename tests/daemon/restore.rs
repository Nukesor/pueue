use anyhow::Result;
use pretty_assertions::assert_eq;

use pueue_lib::network::message::TaskSelection;
use pueue_lib::state::GroupStatus;

use crate::fixtures::*;
use crate::helper::*;

/// The daemon should start in the same state as before shutdown, if no tasks are queued.
/// This function tests for the running state.
#[tokio::test]
async fn test_start_running() -> Result<()> {
    let (settings, _tempdir) = daemon_base_setup()?;
    let mut child = standalone_daemon(&settings.shared).await?;
    let shared = &settings.shared;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(shared).await?);
    wait_for_shutdown(&mut child).await?;

    // Boot it up again
    let mut child = standalone_daemon(&settings.shared).await?;

    // Assert that the group is still running.
    let state = get_state(shared).await?;
    assert_eq!(
        state.groups.get(PUEUE_DEFAULT_GROUP).unwrap().status,
        GroupStatus::Running
    );

    child.kill()?;
    Ok(())
}

/// The daemon should start in the same state as before shutdown, if no tasks are queued.
/// This function tests for the paused state.
#[tokio::test]
async fn test_start_paused() -> Result<()> {
    let (settings, _tempdir) = daemon_base_setup()?;
    let mut child = standalone_daemon(&settings.shared).await?;
    let shared = &settings.shared;

    // This pauses the daemon
    pause_tasks(shared, TaskSelection::All).await?;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(shared).await?);
    wait_for_shutdown(&mut child).await?;

    // Boot it up again
    let mut child = standalone_daemon(&settings.shared).await?;

    // Assert that the group is still paused.
    let state = get_state(shared).await?;
    assert_eq!(
        state.groups.get(PUEUE_DEFAULT_GROUP).unwrap().status,
        GroupStatus::Paused
    );

    child.kill()?;
    Ok(())
}
