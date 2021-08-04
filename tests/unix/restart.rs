use anyhow::Result;
use pueue_lib::network::message::*;

use crate::helper::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
/// Ensure that, by default, restarting a task, creates a new one with the same properties.
async fn test_restart_task() -> Result<()> {
    let (settings, tempdir) = base_setup()?;
    let shared = &settings.shared;
    let _pid = boot_daemon(tempdir.path())?;

    // Add a single task that instantly finishes.
    assert_success(fixtures::add_task(shared, "sleep 0.1", false).await?);

    // Wait for task 0 to finish.
    wait_for_task_condition(shared, 0, |task| task.is_done()).await?;

    // Restart task 0 with an extended sleep command.
    let restart_message = Message::Restart(RestartMessage {
        tasks: vec![TasksToRestart {
            task_id: 0,
            command: "sleep 60".to_string(),
            path: "/tmp/".to_string(),
        }],
        start_immediately: false,
        stashed: false,
    });
    let response = send_message(shared, restart_message).await?;
    println!("{:?}", response);

    // Make sure a new task has been created.
    let state = get_state(shared).await?;
    println!("{:?}", state);
    assert_eq!(state.tasks.len(), 2, "A new task should have been crated");

    // Task 1 should soon be started
    wait_for_task_condition(shared, 1, |task| task.is_running()).await?;

    let state = get_state(shared).await?;
    let task = state.tasks.get(&1).unwrap();
    assert_eq!(task.command, "sleep 60");

    Ok(())
}
