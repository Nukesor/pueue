use anyhow::Result;
use pueue_lib::network::message::*;

use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Add and directly remove a group.
async fn test_add_and_remove() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // Add a new group
    let add_message = Message::Group(GroupMessage::Add("testgroup".to_string()));
    assert_success(send_message(shared, add_message.clone()).await?);

    // Make sure it got added
    let state = get_state(shared).await?;
    assert!(state.groups.contains_key("testgroup"));

    // Try to add the same group again. This should fail
    assert_failure(send_message(shared, add_message).await?);

    // Remove the newly added group
    let remove_message = Message::Group(GroupMessage::Remove("testgroup".to_string()));
    assert_success(send_message(shared, remove_message.clone()).await?);

    // Make sure it got removed
    let state = get_state(shared).await?;
    assert!(!state.groups.contains_key("testgroup"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Users cannot delete the default group.
async fn test_cannot_delete_default() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    let pause_message = Message::Group(GroupMessage::Remove("default".to_string()));
    assert_failure(send_message(shared, pause_message).await?);

    Ok(())
}
