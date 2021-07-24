use anyhow::Result;
use pueue_lib::state::State;

use crate::helper::*;

fn assert_worker_envs(state: &State, task_id: usize, worker: usize, group: &str) {
    let task = state.tasks.get(&task_id).unwrap();
    assert_eq!(task.envs.get("PUEUE_GROUP"), Some(&group.to_string()));
    assert_eq!(task.envs.get("PUEUE_WORKER_ID"), Some(&worker.to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure that the expected worker variables are injected into the tasks' environment variables
/// for a single task on the default queue.
async fn test_single_worker() -> Result<()> {
    better_panic::debug_install();
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // Add some tasks that instantly finish.
    for _ in 0..3 {
        assert_success(fixtures::add_task(shared, "sleep 0.1", false).await?);
    }

    // Wait for the last task to finish.
    wait_for_task_condition(&settings.shared, 2, |task| task.is_done()).await?;

    // All tasks should have the worker id 0, as the tasks are processed sequentially.
    let state = get_state(shared).await?;
    for task_id in 0..3 {
        assert_worker_envs(&state, task_id, 0, "default");
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
    better_panic::debug_install();
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    for _ in 0..5 {
        assert_success(fixtures::add_task_to_group(shared, "sleep 0.2", "test_3").await?);
    }

    // Wait for the last task to finish.
    wait_for_task_condition(&settings.shared, 4, |task| task.is_done()).await?;

    // The first three tasks should have the same worker id's as the task ids.
    let state = get_state(shared).await?;
    for task_id in 0..3 {
        assert_worker_envs(&state, task_id, task_id, "test_3");
    }

    // Task3 gets task0's slot
    assert_worker_envs(&state, 3, 0, "test_3");
    // Task4 gets task1's slot
    assert_worker_envs(&state, 4, 1, "test_3");

    Ok(())
}
