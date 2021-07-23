use std::convert::TryInto;

use anyhow::Result;
use pretty_assertions::assert_eq;

use pueue_lib::network::message::TaskSelection;
use pueue_lib::state::GroupStatus;

use crate::helper::*;

#[tokio::test]
/// The daemon should start in the same state as before shutdown, if no tasks are queued.
/// This function tests for the running state.
async fn test_start_running() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;

    let child = boot_standalone_daemon(tempdir.path())?;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(&shared).await?);
    wait_for_shutdown(child.id().try_into()?)?;

    // Boot it up again
    let mut child = boot_standalone_daemon(tempdir.path())?;

    // Assert that the group is still running.
    let state = get_state(shared).await?;
    assert_eq!(state.groups.get("default").unwrap(), &GroupStatus::Running);

    child.kill()?;
    Ok(())
}

#[tokio::test]
/// The daemon should start in the same state as before shutdown, if no tasks are queued.
/// This function tests for the paused state.
async fn test_start_paused() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;

    let child = boot_standalone_daemon(tempdir.path())?;

    // This pauses the daemon
    pause_tasks(shared, TaskSelection::All).await?;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(&shared).await?);
    wait_for_shutdown(child.id().try_into()?)?;

    // Boot it up again
    let mut child = boot_standalone_daemon(tempdir.path())?;

    // Assert that the group is still paused.
    let state = get_state(shared).await?;
    assert_eq!(state.groups.get("default").unwrap(), &GroupStatus::Paused);

    child.kill()?;
    Ok(())
}
