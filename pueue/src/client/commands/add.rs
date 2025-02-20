use std::{
    borrow::Cow,
    collections::HashMap,
    env::{current_dir, vars},
    path::PathBuf,
};

use chrono::{DateTime, Local};
use pueue_lib::{
    Request, Response,
    client::Client,
    format::format_datetime,
    network::message::{AddRequest, AddedTaskResponse},
};

use super::{follow as follow_cmd, group_or_default, handle_response};
use crate::{client::style::OutputStyle, internal_prelude::*};

#[allow(clippy::too_many_arguments)]
pub async fn add_task(
    client: &mut Client,
    style: &OutputStyle,
    mut command: Vec<String>,
    working_directory: Option<PathBuf>,
    escape: bool,
    start_immediately: bool,
    stashed: bool,
    group: Option<String>,
    delay_until: Option<DateTime<Local>>,
    dependencies: Vec<usize>,
    priority: Option<i32>,
    label: Option<String>,
    print_task_id: bool,
    follow: bool,
) -> Result<()> {
    // Either take the user-specified path or default to the current working directory.
    // This will give errors if connecting over TCP/TLS to a remote host that doesn't
    // have the same directory structure as the client
    let path = working_directory
        .as_ref()
        .map(|path| Ok(path.clone()))
        .unwrap_or_else(current_dir)?;

    // The user can request to escape any special shell characters in all parameter
    // strings before we concatenated them to a single string.
    if escape {
        command = command
            .iter()
            .map(|parameter| shell_escape::escape(Cow::from(parameter)).into_owned())
            .collect();
    }

    // Add the message to the daemon.
    let message = Request::Add(AddRequest {
        command: command.join(" "),
        path,
        // Catch the current environment for later injection into the task's process.
        envs: HashMap::from_iter(vars()),
        start_immediately,
        stashed,
        group: group_or_default(&group),
        enqueue_at: delay_until,
        dependencies,
        priority,
        label,
    });
    client.send_request(message).await?;

    // Get the response from the daemon.
    let response = client.receive_response().await?;

    // Make sure the task has been added, otherwise handle the response and return.
    let Response::AddedTask(AddedTaskResponse {
        task_id,
        enqueue_at,
        group_is_paused,
    }) = response
    else {
        handle_response(style, response)?;
        return Ok(());
    };

    let mut output = if print_task_id {
        // Only print the task id if that was requested.
        format!("{task_id}")
    } else if let Some(enqueue_at) = enqueue_at {
        let enqueue_at = format_datetime(&client.settings, &enqueue_at);
        format!("New task added (id {task_id}). It will be enqueued at {enqueue_at}")
    } else {
        format!("New task added (id {task_id}).")
    };

    // Also notify the user if the task's group is paused
    if !print_task_id && group_is_paused && !follow {
        output.push_str("\nThe group of this task is currently paused!");
    }

    // If we were to follow the task immediately, print the task info to `stderr` so
    // that the actual log output may be piped into another command.
    if follow {
        eprintln!("{output}");
    } else {
        println!("{output}");
    }

    if follow {
        follow_cmd(client, style, Some(task_id), None).await?;
    }

    Ok(())
}
