use ::anyhow::{bail, Result};
use ::async_std::net::TcpStream;

use ::pueue::message::*;
use ::pueue::protocol::*;
use ::pueue::task::{Task, TaskStatus};

use crate::commands::edit::edit_line;
use crate::commands::get_state;

/// When Restarting tasks, the remote state is queried and a AddMessage
/// is create from the existing task in the state.
///
/// This is done on the client-side, so we can easily edit the task before restarting it.
pub async fn restart(
    socket: &mut TcpStream,
    task_ids: Vec<usize>,
    start_immediately: bool,
    stashed: bool,
    edit_command: bool,
    edit_path: bool,
) -> Result<()> {
    let new_status = if stashed {
        TaskStatus::Stashed
    } else {
        TaskStatus::Queued
    };

    let mut state = get_state(socket).await?;
    let (matching, mismatching) = state.tasks_in_statuses(vec![TaskStatus::Done], Some(task_ids));

    // Go through all Done commands we found and restart them
    for task_id in &matching {
        let task = state.tasks.get(task_id).unwrap();
        let mut new_task = Task::from_task(task);
        new_task.status = new_status.clone();

        // Path and command can be edited, if the use specified the -e or -p flag.
        let mut command = task.command.clone();
        let mut path = task.path.clone();
        if edit_command {
            command = edit_line(&command)?
        };
        if edit_path {
            path = edit_line(&path)?;
        }

        // Create a AddMessage to add the task to the daemon from the
        // updated info and the old task.
        let add_task_message = Message::Add(AddMessage {
            command,
            path,
            envs: task.envs.clone(),
            start_immediately,
            stashed,
            group: task.group.clone(),
            enqueue_at: None,
            dependencies: Vec::new(),
            ignore_aliases: true,
        });

        // Send the cloned task to the daemon and abort on any Failure messages.
        send_message(add_task_message, socket).await?;
        if let Message::Failure(message) = receive_message(socket).await? {
            bail!(message);
        };
    }

    if !matching.is_empty() {
        println!("Restarted tasks: {:?}", matching);
    }
    if !mismatching.is_empty() {
        println!("Couldn't restart tasks: {:?}", mismatching);
    }

    Ok(())
}
