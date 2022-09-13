use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Duration, Local};
use pueue_daemon_lib::state_helper::save_state;

use pueue_lib::settings::Settings;
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

/// Build and save a state with some pre-build tasks we can use to test our filters.
/// The state is saved and created before the daemon is started.
fn build_test_state(settings: &Settings) -> Result<()> {
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

    save_state(&state, settings)?;

    Ok(())
}

/// Test that the normal status command works as expected.
/// Calling `pueue` without any subcommand is equivalent of using `status`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn column_filter() -> Result<()> {
    let (settings, tempdir) = daemon_base_setup()?;
    build_test_state(&settings)?;
    let daemon = daemon_with_settings(settings, tempdir).await?;
    let shared = &daemon.settings.shared;

    // Add a task and wait until it finishes.
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    let output = run_client_command(shared, &["status"]).await?;

    let context = get_task_context(&daemon.settings).await?;
    assert_stdout_matches("status__default_status", output.stdout, context)?;

    Ok(())
}
