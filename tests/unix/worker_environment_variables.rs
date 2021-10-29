use anyhow::Result;

use pueue_lib::settings::PUEUE_DEFAULT_GROUP;

use crate::fixtures::*;
use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure that the expected worker variables are injected into the tasks' environment variables
/// for a single task on the default queue.
async fn test_single_worker() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // Add some tasks that instantly finish.
    for _ in 0..3 {
        assert_success(add_env_task(shared, "sleep 0.1").await?);
    }

    // Wait for the last task to finish.
    wait_for_task_condition(shared, 2, |task| task.is_done()).await?;

    // All tasks should have the worker id 0, as the tasks are processed sequentially.
    let state = get_state(shared).await?;
    for task_id in 0..3 {
        assert_worker_envs(shared, &state, task_id, 0, PUEUE_DEFAULT_GROUP).await?;
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure the correct workers are used when having multiple slots.
///
/// Slots should be properly freed and re-used.
/// Add some tasks to a group with three slots:
///
/// Task0-2 should be started in quick succession.
/// Task3 should take Task0's slot once it's finished.
/// Task4 should take Task1's slot.
async fn test_multiple_worker() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // Spawn three tasks and wait for them
    for _ in 0..3 {
        assert_success(add_env_task_to_group(shared, "sleep 0.1", "test_3").await?);
    }
    wait_for_task_condition(shared, 2, |task| task.is_done()).await?;

    // The first three tasks should have the same worker id's as the task ids.
    let state = get_state(shared).await?;
    for task_id in 0..3 {
        assert_worker_envs(shared, &state, task_id, task_id, "test_3").await?;
    }

    // Spawn to more tasks and wait for them
    for _ in 0..2 {
        assert_success(add_env_task_to_group(shared, "sleep 0.1", "test_3").await?);
    }
    wait_for_task_condition(shared, 4, |task| task.is_done()).await?;

    let state = get_state(shared).await?;
    // Task3 gets worker0
    assert_worker_envs(shared, &state, 3, 0, "test_3").await?;
    // Task4 gets worker1
    assert_worker_envs(shared, &state, 4, 1, "test_3").await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure the worker pools are properly initialized when maually adding a new group.
async fn test_worker_for_new_pool() -> Result<()> {
    let daemon = daemon()?;
    let shared = &daemon.settings.shared;

    // Add a new group
    add_group_with_slots(shared, "testgroup", 1).await?;

    // Add some tasks that instantly finish.
    assert_success(add_env_task_to_group(shared, "sleep 0.1", "testgroup").await?);
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // The task should have the correct worker id + group.
    let state = get_state(shared).await?;
    assert_worker_envs(shared, &state, 0, 0, "testgroup").await?;

    Ok(())
}
