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
        tasks: TaskSelection::All,
        children: false,
        signal: None,
    }), true
)]
#[case(
    Message::Kill(KillMessage {
        tasks: TaskSelection::Group("default".into()),
        children: false,
        signal: None,
    }), true
)]
#[case(
    Message::Kill(KillMessage {
        tasks: TaskSelection::TaskIds(vec![0, 1, 2]),
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
        assert_success(fixtures::add_task(shared, "sleep 60", true).await?);
    }
    // Wait until all tasks are running
    for id in 0..3 {
        wait_for_task_condition(shared, id, |task| task.is_running()).await?;
    }

    // Send the kill message
    send_message(shared, kill_message).await?;

    // Make sure all tasks get killed
    for id in 0..3 {
        wait_for_task_condition(shared, id, |task| {
            matches!(task.status, TaskStatus::Done(TaskResult::Killed))
        })
        .await?;
    }

    // Groups should be paused in specific modes.
    if group_should_pause {
        let state = get_state(shared).await?;
        assert_eq!(state.groups.get("default").unwrap(), &GroupStatus::Paused);
    }

    Ok(())
}
