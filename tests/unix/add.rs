use anyhow::Result;
use pueue_lib::task::*;

use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if adding a normal task works as intended.
async fn test_normal_add() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // Add a task that instantly finishes
    assert_success(fixtures::add_task(shared, "sleep 0.01", true).await?);

    // Wait until the task finished and get state
    wait_for_task_condition(&settings.shared, 0, |task| task.is_done()).await?;

    // The task finished succesfully
    assert_eq!(
        get_task_status(shared, 0).await?,
        TaskStatus::Done(TaskResult::Success)
    );

    Ok(())
}
