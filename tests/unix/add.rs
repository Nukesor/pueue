use anyhow::Result;
use pueue_lib::network::message::*;
use pueue_lib::task::*;

use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if adding a normal task works as intended.
async fn test_normal_add() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // Add a task that instantly finishes
    let response = fixtures::add_task(shared, "sleep 0.01", true).await?;
    assert!(matches!(response, Message::Success(_)));

    // Wait until the task finished and get state
    wait_for_status(&settings.shared, 0, TaskStatus::Done).await?;
    let state = get_state(shared).await?;

    // The task finished succesfully
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.status, TaskStatus::Done);
    assert_eq!(task.result, Some(TaskResult::Success));

    Ok(())
}
