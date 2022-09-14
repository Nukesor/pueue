use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Duration, Local};
use pueue_daemon_lib::state_helper::save_state;

use pueue_lib::state::{State, PUEUE_DEFAULT_GROUP};
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::fixtures::*;
use crate::helper::*;

/// A small helper function to reduce a bit of boilerplate.
fn build_task() -> Task {
    Task::new(
        "sleep 60".to_owned(),
        PathBuf::from("/tmp"),
        HashMap::new(),
        PUEUE_DEFAULT_GROUP.to_owned(),
        TaskStatus::Queued,
        Vec::new(),
        None,
    )
}

/// Initialize a daemon which already contains a predefined list of tasks in various states.
async fn daemon_with_test_state() -> Result<PueueDaemon> {
    // Get the base setup for the daemon
    let (settings, tempdir) = daemon_base_setup()?;

    // ------ Inert tasks -------
    // Build and save a state with some pre-build tasks we can use to test our filters.
    // The state is saved and created before the daemon is started.

    let mut state = State::new();

    // Failed task
    let mut failed = build_task();
    failed.id = 0;
    failed.status = TaskStatus::Done(TaskResult::Failed(255));
    failed.start = Some(Local::now() - Duration::days(1));
    failed.end = Some(Local::now() - Duration::days(1) + Duration::minutes(1));
    state.tasks.insert(failed.id, failed);

    // Successful task
    let mut successful = build_task();
    successful.id = 1;
    successful.status = TaskStatus::Done(TaskResult::Success);
    successful.start = Some(Local::now() - Duration::days(2));
    successful.end = Some(Local::now() - Duration::days(2) + Duration::minutes(1));
    state.tasks.insert(successful.id, successful);

    // Stashed task
    let mut successful = build_task();
    successful.status = TaskStatus::Stashed { enqueue_at: None };
    successful.id = 2;
    state.tasks.insert(successful.id, successful);

    // Scheduled task
    let mut successful = build_task();
    successful.status = TaskStatus::Stashed {
        enqueue_at: Some(Local::now() + Duration::hours(1)),
    };
    successful.id = 3;
    state.tasks.insert(successful.id, successful);

    // Save the state in our temporary directory. This makes it readable by the daemon.
    save_state(&state, &settings)?;

    // ------ Daemon setup -------
    // Start the daemon. It will restore the state we just saved.
    let daemon = daemon_with_settings(settings, tempdir).await?;
    let shared = &daemon.settings.shared;

    // ------ Live tasks -------
    // Now we have to add some tasks that need to be added live.

    // Running task
    assert_success(add_task(shared, "sleep 60", false).await?);

    // 2 Queued tasks
    assert_success(add_task(shared, "sleep 60", false).await?);
    assert_success(add_task(shared, "sleep 60", false).await?);

    Ok(daemon)
}

/// This is a default `status` call without any paramaters.
/// This only exists to ensure the baseline behavior of our test state.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filter_default() -> Result<()> {
    let daemon = daemon_with_test_state().await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["status"]).await?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("filter__default_status", output.stdout, context)?;

    Ok(())
}
