use anyhow::Result;
use pretty_assertions::assert_eq;

use pueue_lib::task::*;

use crate::fixtures::*;
use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test that multiple groups with multiple slots work.
///
/// For each group, Pueue should start tasks until all slots are filled.
async fn test_parallel_tasks() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    // ---- First group ----
    // Add a new group with 3 slots
    add_group_with_slots(shared, "testgroup_3", 3).await?;

    // Add 5 tasks to this group, only 3 should be started.
    for _ in 0..5 {
        assert_success(add_task_to_group(shared, "sleep 60", "testgroup_3").await?);
    }

    // Ensure those three tasks are started.
    for task_id in 0..3 {
        wait_for_task_condition(shared, task_id, |task| task.is_running()).await?;
    }

    // Tasks 4-5 should still be queued
    let state = get_state(shared).await?;
    for task_id in 3..5 {
        let task = state.tasks.get(&task_id).unwrap();
        assert_eq!(task.status, TaskStatus::Queued);
    }

    // ---- Second group ----
    // Add another group with 2 slots
    add_group_with_slots(shared, "testgroup_2", 2).await?;

    // Add another 5 tasks to this group, only 2 should be started.
    for _ in 0..5 {
        assert_success(add_task_to_group(shared, "sleep 60", "testgroup_2").await?);
    }

    // Ensure only two tasks are started.
    for task_id in 5..7 {
        wait_for_task_condition(shared, task_id, |task| task.is_running()).await?;
    }

    // Tasks 8-10 should still be queued
    let state = get_state(shared).await?;
    for task_id in 7..10 {
        let task = state.tasks.get(&task_id).unwrap();
        assert_eq!(task.status, TaskStatus::Queued);
    }
    Ok(())
}
