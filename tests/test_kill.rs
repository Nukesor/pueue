use anyhow::Result;
use pueue_lib::network::message::*;
use pueue_lib::task::*;
use rstest::rstest;

mod helper;

use helper::*;

#[cfg(target_os = "linux")]
#[rstest]
#[case(
    Message::Kill(KillMessage {
        task_ids: vec![],
        group: "default".into(),
        all: true,
        children: false,
        signal: None,
    })
)]
#[case(
    Message::Kill(KillMessage {
        task_ids: vec![],
        group: "default".into(),
        all: false,
        children: false,
        signal: None,
    })
)]
#[case(
    Message::Kill(KillMessage {
        task_ids: vec![0, 1, 2],
        group: "default".into(),
        all: false,
        children: false,
        signal: None,
    })
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if killing running tasks works as intended.
///
/// We test different ways of killing those tasks.
/// - Via the --all flag, which just kills everything.
/// - Via the --group flag, which just kills everything in the default group.
/// - Via specific ids.
async fn test_kill_tasks(#[case] kill_message: Message) -> Result<()> {
    let (settings, tempdir) = helper::base_setup()?;
    let shared = &settings.shared;
    let _pid = helper::boot_daemon(tempdir.path())?;

    // Add multiple tasks and start them immediately
    for _ in 0..3 {
        let response = fixtures::add_task(shared, "sleep 60", true).await?;
        assert!(matches!(response, Message::Success(_)));
    }
    // Wait until all tasks are running
    for id in 0..3 {
        wait_for_status(shared, id, TaskStatus::Running).await?;
    }

    // Send the kill message
    send_message(shared, kill_message).await?;

    // Wait until the task are finished
    for id in 0..3 {
        wait_for_status(shared, id, TaskStatus::Done).await?;
    }

    // Make sure the tasks have been killed
    let state = get_state(shared).await?;
    for id in 0..3 {
        let task = state.tasks.get(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Done);
        assert_eq!(task.result, Some(TaskResult::Killed));
    }

    Ok(())
}
