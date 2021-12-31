use anyhow::{bail, Result};

use pueue_lib::network::message::*;
use pueue_lib::settings::Shared;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::*;

use crate::fixtures::*;
use crate::helper::*;

async fn create_edited_task(shared: &Shared) -> Result<EditResponseMessage> {
    // Add a task
    assert_success(add_task(shared, "ls", false).await?);

    // The task should now be queued
    assert_eq!(get_task_status(shared, 0).await?, TaskStatus::Queued);

    // Send a request to edit that task
    let response = send_message(shared, Message::EditRequest(0)).await?;
    if let Message::EditResponse(payload) = response {
        Ok(payload)
    } else {
        bail!("Didn't receive EditResponse after requesting edit.")
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if adding a normal task works as intended.
async fn test_edit_flow() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // Pause the daemon. That way the command won't be started.
    pause_tasks(shared, TaskSelection::All).await?;
    wait_for_group_status(shared, PUEUE_DEFAULT_GROUP, GroupStatus::Paused).await?;

    let response = create_edited_task(shared).await?;
    assert_eq!(response.task_id, 0);
    assert_eq!(response.command, "ls");
    assert_eq!(response.path, daemon.tempdir.path().to_string_lossy());

    // Task should be locked, after the request for editing succeeded.
    assert_eq!(get_task_status(shared, 0).await?, TaskStatus::Locked);

    // You cannot start a locked task. It should still be locked afterwards.
    start_tasks(shared, TaskSelection::TaskIds(vec![0])).await?;
    assert_eq!(get_task_status(shared, 0).await?, TaskStatus::Locked);

    // Send the final message of the protocol and actually change the task.
    let response = send_message(
        shared,
        Message::Edit(EditMessage {
            task_id: 0,
            command: "ls -ahl".into(),
            path: "/tmp".into(),
        }),
    )
    .await?;
    assert_success(response);

    // Make sure the task has been changed and enqueued.
    let task = get_task(shared, 0).await?;
    assert_eq!(task.command, "ls -ahl");
    assert_eq!(task.path, "/tmp");
    assert_eq!(task.status, TaskStatus::Queued);

    Ok(())
}
