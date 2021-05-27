use anyhow::Result;
use pueue_lib::network::message::*;
use pueue_lib::task::*;

mod helper;

use helper::*;

#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if adding a normal task works as intended.
async fn test_start_tasks() -> Result<()> {
    let (settings, tempdir) = helper::base_setup()?;
    let shared = &settings.shared;
    let _pid = helper::boot_daemon(tempdir.path())?;

    // Add multiple tasks and start them immediately
    for _ in 0..3 {
        let response = fixtures::add_task(shared, "sleep 60", true).await?;
        assert!(matches!(response, Message::Success(_)));
    }
    // Wait until all tasks are running
    for id in 0..3 {
        wait_for_status(shared, id, TaskStatus::Running).await?;
    }

    // Pause the whole daemon and wait until all tasks are paused
    pause_daemon(shared).await?;
    for id in 0..3 {
        wait_for_status(shared, id, TaskStatus::Paused).await?;
    }

    // Continue all tasks again
    continue_daemon(shared).await?;
    // Wait until the task are running again
    for id in 0..3 {
        wait_for_status(shared, id, TaskStatus::Running).await?;
    }

    // All tasks should be up and running
    let state = get_state(shared).await?;
    for id in 0..3 {
        let task = state.tasks.get(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Running);
        assert_eq!(task.result, None);
    }

    shutdown(shared).await?;
    Ok(())
}
