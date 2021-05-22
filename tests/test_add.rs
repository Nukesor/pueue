use std::collections::HashMap;

use anyhow::Result;
use pueue_lib::network::message::*;
use pueue_lib::task::*;

mod fixtures;
mod helper;

#[async_std::test]
/// Test if adding a normal task works as intended.
async fn test_normal_add() -> Result<()> {
    let (settings, tempdir) = helper::base_setup()?;

    let _pid = helper::start_daemon(tempdir.path())?;

    let message = Message::Add(AddMessage {
        command: "sleep 0.01".into(),
        path: "/tmp".into(),
        envs: HashMap::new(),
        start_immediately: false,
        stashed: false,
        group: "default".into(),
        enqueue_at: None,
        dependencies: vec![],
        label: None,
        print_task_id: false,
    });
    let response = fixtures::send_message(&settings.shared, message).await?;
    assert!(matches!(response, Message::Success(_)));

    // Slep a little so the taskhandler can spawn the program
    helper::sleep_ms(500);

    let state = fixtures::get_state(&settings.shared).await?;
    fixtures::shutdown(&settings.shared).await?;

    // A task exists
    assert_eq!(state.tasks.len(), 2);

    // The task finished succesfully
    let task = state.tasks.get(&0).unwrap();
    assert_eq!(task.status, TaskStatus::Done);
    assert_eq!(task.result, Some(TaskResult::Success));

    Ok(())
}
