use anyhow::Result;

use pueue_lib::network::message::*;

use crate::fixtures::*;
use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Add and directly remove a group.
async fn test_add_and_remove() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // Add a new group
    add_group_with_slots(shared, "testgroup", 1).await?;

    // Try to add the same group again. This should fail
    let add_message = Message::Group(GroupMessage::Add {
        name: "testgroup".to_string(),
        parallel_tasks: None,
    });
    assert_failure(send_message(shared, add_message).await?);

    // Remove the newly added group and wait for the deletion to be processed.
    let remove_message = Message::Group(GroupMessage::Remove("testgroup".to_string()));
    assert_success(send_message(shared, remove_message.clone()).await?);
    wait_for_group_absence(shared, "testgroup").await?;

    // Make sure it got removed
    let state = get_state(shared).await?;
    assert!(!state.groups.contains_key("testgroup"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Users cannot delete the default group.
async fn test_cannot_delete_default() -> Result<()> {
    let daemon = daemon()?;

    let pause_message = Message::Group(GroupMessage::Remove(PUEUE_DEFAULT_GROUP.to_string()));
    assert_failure(send_message(&daemon.settings.shared, pause_message).await?);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Users cannot delete a non-existing group.
async fn test_cannot_delete_non_existing() -> Result<()> {
    let daemon = daemon()?;

    let pause_message = Message::Group(GroupMessage::Remove("doesnt_exist".to_string()));
    assert_failure(send_message(&daemon.settings.shared, pause_message).await?);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Groups with tasks shouldn't be able to be removed.
async fn test_cannot_delete_group_with_tasks() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // Add a new group
    add_group_with_slots(shared, "testgroup", 1).await?;

    // Add a task
    assert_success(add_task_to_group(shared, "ls", "testgroup").await?);
    wait_for_task_condition(&daemon.settings.shared, 0, |task| task.is_done()).await?;

    // We shouldn't be capable of removing that group
    let pause_message = Message::Group(GroupMessage::Remove("testgroup".to_string()));
    assert_failure(send_message(shared, pause_message).await?);

    // Remove the task from the group
    let remove_message = Message::Remove(vec![0]);
    send_message(shared, remove_message).await?;

    // Removal should now work.
    let pause_message = Message::Group(GroupMessage::Remove("testgroup".to_string()));
    assert_success(send_message(shared, pause_message).await?);

    Ok(())
}
