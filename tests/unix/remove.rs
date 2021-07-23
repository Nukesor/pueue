use anyhow::Result;
use pueue_lib::network::message::*;

use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Ensure that clean only removes finished tasks
async fn test_normal_clean() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // We'll add some tasks. Some statuses will be adjusted lateron.
    // 0 -> failed
    // 1 -> success
    // 2 -> running
    // 3 -> paused
    // 4 -> queued
    // 5 -> stashed
    for command in vec!["failing", "ls", "sleep 60", "sleep 60", "ls", "ls"] {
        assert_success(fixtures::add_task(shared, command, false).await?);
    }
    // Wait for task2 to start. This implies task[0,1] being finished.
    wait_for_task_condition(shared, 2, |task| task.is_running()).await?;

    // Explicitely start task3, wait for it to start and directly pause it.
    start_tasks(shared, TaskSelection::TaskIds(vec![3])).await?;
    wait_for_task_condition(shared, 3, |task| task.is_running()).await?;

    pause_tasks(&shared, TaskSelection::TaskIds(vec![3])).await?;

    // Stash task 5
    let pause_message = Message::Stash(vec![5]);
    send_message(shared, pause_message).await?;

    let remove_message = Message::Remove(vec![0, 1, 2, 3, 4, 5]);
    send_message(shared, remove_message).await?;

    // Every task that isn't currently running can be removed
    let state = get_state(shared).await?;
    assert!(!state.tasks.contains_key(&0));
    assert!(!state.tasks.contains_key(&1));
    assert!(state.tasks.contains_key(&2));
    assert!(state.tasks.contains_key(&3));
    assert!(!state.tasks.contains_key(&4));
    assert!(!state.tasks.contains_key(&5));

    Ok(())
}
