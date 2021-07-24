use anyhow::Result;

use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Make sure that the expected worker variables are injected into the tasks' environment variables
/// for a single task on the default queue.
async fn test_single_worker() -> Result<()> {
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
        let task = state.tasks.get(&task_id).unwrap();
        assert_eq!(task.envs.get("PUEUE_GROUP"), Some(&"default".to_string()));
        assert_eq!(task.envs.get("PUEUE_WORKER_ID"), Some(&"0".to_string()));
    }

    Ok(())
}
