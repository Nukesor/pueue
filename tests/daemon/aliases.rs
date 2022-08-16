use std::collections::HashMap;

use anyhow::Result;

use pueue_lib::task::*;

use crate::fixtures::*;
use crate::helper::*;

/// Test that using aliases when adding task normally works.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_add_with_alias() -> Result<()> {
    let daemon = daemon().await?;
    let shared = &daemon.settings.shared;

    let mut aliases = HashMap::new();
    aliases.insert("rofl".into(), "echo".into());
    create_test_alias_file(daemon.tempdir.path(), aliases)?;

    // Add a task whose command should be replaced by an alias
    assert_success(add_task(shared, "rofl test", false).await?);

    // Wait until the task finished and get state
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let task = get_task(shared, 0).await?;

    // The task finished successfully and its command has replaced the alias.
    assert_eq!(task.status, TaskStatus::Done(TaskResult::Success));
    assert_eq!(task.command, "echo test");
    assert_eq!(task.original_command, "rofl test");

    // Make sure we see an actual "test" in the output.
    // This ensures that we really called "echo".
    let log = get_task_log(shared, 0, None).await?;
    assert_eq!(log, "test\n");

    Ok(())
}
