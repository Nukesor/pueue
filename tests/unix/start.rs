use anyhow::Result;
use pueue_lib::network::message::*;
use pueue_lib::task::*;
use rstest::rstest;

use crate::helper::*;

#[rstest]
#[case(
    Message::Start(StartMessage {
        tasks: TaskSelection::All,
        children: false,
    })
)]
#[case(
    Message::Start(StartMessage {
        tasks: TaskSelection::Group("default".into()),
        children: false,
    })
)]
#[case(
    Message::Start(StartMessage {
        tasks: TaskSelection::TaskIds(vec![0, 1, 2]),
        children: false,
    })
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if explicitely starting tasks and resuming tasks works as intended.
///
/// We test different ways of resumes tasks.
/// - Via the --all flag, which resumes everything.
/// - Via the --group flag, which resumes everything in a specific group (in our case 'default').
/// - Via specific ids.
async fn test_start_tasks(#[case] start_message: Message) -> Result<()> {
    let (settings, _tempdir, _pid) = threaded_setup()?;
    let shared = &settings.shared;

    // Add multiple tasks only a single one will be started by default
    for _ in 0..3 {
        assert_success(fixtures::add_task(shared, "sleep 60", false).await?);
    }
    // Wait for task 0 to start on its own.
    // We have to do this, otherwise we'll start task 1/2 beforehand, which prevents task 0 to be
    // started on its own.
    wait_for_task_condition(shared, 0, |task| task.is_running()).await?;

    // Start tasks 1 and 2 manually
    start_tasks(shared, TaskSelection::TaskIds(vec![1, 2])).await?;

    // Wait until all tasks are running
    for id in 0..3 {
        wait_for_task_condition(shared, id, |task| task.is_running()).await?;
    }

    // Pause the whole daemon and wait until all tasks are paused
    pause_tasks(shared, TaskSelection::All).await?;
    for id in 0..3 {
        wait_for_task_condition(shared, id, |task| matches!(task.status, TaskStatus::Paused))
            .await?;
    }

    // Send the kill message
    send_message(shared, start_message).await?;

    // Ensure all tasks are running
    for id in 0..3 {
        wait_for_task_condition(shared, id, |task| task.is_running()).await?;
    }
    Ok(())
}
