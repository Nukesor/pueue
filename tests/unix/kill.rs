use anyhow::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;

use pueue_lib::network::message::*;
use pueue_lib::state::GroupStatus;
use pueue_lib::task::*;

use crate::helper::*;

#[rstest]
#[case(
    Message::Kill(KillMessage {
        task_ids: vec![],
        group: "default".into(),
        all: true,
        children: false,
        signal: None,
    }), true
)]
#[case(
    Message::Kill(KillMessage {
        task_ids: vec![],
        group: "default".into(),
        all: false,
        children: false,
        signal: None,
    }), true
)]
#[case(
    Message::Kill(KillMessage {
        task_ids: vec![0, 1, 2],
        group: "default".into(),
        all: false,
        children: false,
        signal: None,
    }), false
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Test if killing running tasks works as intended.
///
/// We test different ways of killing those tasks.
/// - Via the --all flag, which just kills everything.
/// - Via the --group flag, which just kills everything in the default group.
/// - Via specific ids.
///
/// If a whole group or everything is killed, the respective groups should also be paused!
/// This is security measure to prevent unwanted task execution in an emergency.
async fn test_kill_tasks(
    #[case] kill_message: Message,
    #[case] group_should_pause: bool,
) -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // Add multiple tasks and start them immediately
    for _ in 0..3 {
        let response = fixtures::add_task(shared, "sleep 60", true).await?;
        assert!(matches!(response, Message::Success(_)));
    }
    // Wait until all tasks are running
    for id in 0..3 {
        wait_for_task_condition(shared, id, |task| {
            matches!(task.status, TaskStatus::Running)
        })
        .await?;
    }

    // Send the kill message
    send_message(shared, kill_message).await?;

    // Wait until the task are finished
    for id in 0..3 {
        wait_for_task_condition(shared, id, |task| {
            matches!(task.status, TaskStatus::Done(_))
        })
        .await?;
    }

    // Make sure the tasks have been killed
    let state = get_state(shared).await?;
    for id in 0..3 {
        let task = state.tasks.get(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Done(TaskResult::Killed));
    }

    // Groups should be paused in specific modes.
    if group_should_pause {
        assert_eq!(state.groups.get("default").unwrap(), &GroupStatus::Paused);
    }

    Ok(())
}
