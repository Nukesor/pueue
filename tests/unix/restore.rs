use std::convert::TryInto;
use std::path::PathBuf;

use anyhow::Result;
use pretty_assertions::assert_eq;

use pueue_lib::network::message::TaskSelection;
use pueue_lib::state::GroupStatus;

use crate::fixtures::*;
use crate::helper::*;

#[tokio::test]
/// The daemon should start in the same state as before shutdown, if no tasks are queued.
/// This function tests for the running state.
async fn test_start_running() -> Result<()> {
    let (settings, tempdir) = daemon_base_setup()?;
    let child = standalone_daemon(tempdir.path()).await?;
    let shared = &settings.shared;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(shared).await?);
    wait_for_shutdown(child.id().try_into()?)?;

    // Boot it up again
    let mut child = standalone_daemon(tempdir.path()).await?;

    // Assert that the group is still running.
    let state = get_state(shared).await?;
    assert_eq!(
        state.groups.get(PUEUE_DEFAULT_GROUP).unwrap().status,
        GroupStatus::Running
    );

    child.kill()?;
    Ok(())
}

#[tokio::test]
/// The daemon should start in the same state as before shutdown, if no tasks are queued.
/// This function tests for the paused state.
async fn test_start_paused() -> Result<()> {
    let (settings, tempdir) = daemon_base_setup()?;
    let child = standalone_daemon(tempdir.path()).await?;
    let shared = &settings.shared;

    // This pauses the daemon
    pause_tasks(shared, TaskSelection::All).await?;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(shared).await?);
    wait_for_shutdown(child.id().try_into()?)?;

    // Boot it up again
    let mut child = standalone_daemon(tempdir.path()).await?;

    // Assert that the group is still paused.
    let state = get_state(shared).await?;
    assert_eq!(
        state.groups.get(PUEUE_DEFAULT_GROUP).unwrap().status,
        GroupStatus::Paused
    );

    child.kill()?;
    Ok(())
}

#[tokio::test]
/// The daemon will load new settings, when restoring a previous state.
async fn test_load_config() -> Result<()> {
    let (mut settings, tempdir) = daemon_base_setup()?;
    let child = standalone_daemon(tempdir.path()).await?;

    // Kill the daemon and wait for it to shut down.
    assert_success(shutdown_daemon(&settings.shared).await?);
    wait_for_shutdown(child.id().try_into()?)?;

    // Change the settings and save it to disk
    settings.client.dark_mode = true;
    settings.daemon.callback = Some("This is a test".to_string());
    settings.shared.daemon_key = PathBuf::from("/tmp/daemon.key");
    settings.save(&Some(tempdir.path().join("pueue.yml")))?;

    let mut child = standalone_daemon(tempdir.path()).await?;

    // Get the new state and make sure the settings actually changed.
    let state = get_state(&settings.shared).await?;
    assert_eq!(state.settings.daemon.callback, settings.daemon.callback);
    assert_eq!(state.settings.shared.daemon_key, settings.shared.daemon_key);
    assert_eq!(state.settings.client.dark_mode, settings.client.dark_mode);

    child.kill()?;
    Ok(())
}
