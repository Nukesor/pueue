use anyhow::Result;
use pueue_lib::network::message::*;

use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Ensure that clean only removes finished tasks
async fn test_normal_clean() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // This should result in one failed, one finished, one running and one queued task.
    for command in vec!["failing", "ls", "sleep 60", "ls"] {
        assert_success(fixtures::add_task(shared, command, false).await?);
    }
    // Wait for task2 to start. This implies task[0,1] being finished.
    wait_for_task_condition(shared, 2, |task| task.is_running()).await?;

    // Send the clean message
    let clean_message = CleanMessage {
        successful_only: false,
    };
    send_message(shared, Message::Clean(clean_message)).await?;

    // Assert that task 0 and 1 have both been removed
    let state = get_state(shared).await?;
    assert!(!state.tasks.contains_key(&0));
    assert!(!state.tasks.contains_key(&1));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Ensure only successful tasks are removed, if the `-s` flag is set.
async fn test_successful_only_clean() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // This should result in one failed, one finished, one running and one queued task.
    for command in vec!["failing", "ls"] {
        assert_success(fixtures::add_task(shared, command, false).await?);
    }
    // Wait for task2 to start. This implies task[0,1] being finished.
    wait_for_task_condition(shared, 1, |task| task.is_done()).await?;

    // Send the clean message
    let clean_message = CleanMessage {
        successful_only: true,
    };
    send_message(shared, Message::Clean(clean_message)).await?;

    // Assert that task 0 is still there, as it failed.
    let state = get_state(shared).await?;
    assert!(state.tasks.contains_key(&0));
    // Task 1 should have been removed.
    assert!(!state.tasks.contains_key(&1));

    Ok(())
}
