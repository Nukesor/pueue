//! This module contains the logic for all non-trivial commands, such as `follow`, `restart`,
//! `wait`, etc.
//!
//! "non-trivial" vaguely means that we, for instance, have to do additional requests to the
//! daemon, open some files on the filesystem, edit files and so on.
//! All commands that cannot be simply handled by handling requests or using `pueue_lib`.

use std::io::{stdin, stdout, Write};

use pueue_lib::{
    network::message::{Request, Response, TaskSelection},
    state::{State, PUEUE_DEFAULT_GROUP},
    task::Task,
};

use crate::internal_prelude::*;

mod add;
mod clean;
mod edit;
mod enqueue;
mod env;
mod follow;
mod group;
mod kill;
mod log;
mod parallel;
mod pause;
mod remove;
mod reset;
mod restart;
mod send;
mod shutdown;
mod start;
mod stash;
mod state;
mod switch;
mod wait;

use add::add_task;
use clean::clean;
use edit::edit;
use enqueue::enqueue;
use env::env;
use follow::follow;
use group::group;
use kill::kill;
use log::print_logs;
use parallel::parallel;
use pause::pause;
use remove::remove;
use reset::reset;
use restart::restart;
use send::send;
use shutdown::shutdown;
use start::start;
use stash::stash;
use state::{format_state, state};
use switch::switch;
use wait::wait;
pub use wait::WaitTargetStatus;

use super::{
    cli::SubCommand,
    client::Client,
    display_helper::{print_error, print_success},
    style::OutputStyle,
};

/// This is a small helper which determines a task selection depending on
/// given commandline parameters.
/// I.e. whether the default group, a set of tasks or a specific group should be selected.
/// `start`, `pause` and `kill` can target either of these three selections.
///
/// If no parameters are given, it returns to the default group.
pub fn selection_from_params(
    all: bool,
    group: Option<String>,
    task_ids: Vec<usize>,
) -> TaskSelection {
    if all {
        TaskSelection::All
    } else if let Some(group) = group {
        TaskSelection::Group(group)
    } else if !task_ids.is_empty() {
        TaskSelection::TaskIds(task_ids)
    } else {
        TaskSelection::Group(PUEUE_DEFAULT_GROUP.into())
    }
}

/// This is a small helper which either returns a given group or the default group.
fn group_or_default(group: &Option<String>) -> String {
    group
        .clone()
        .unwrap_or_else(|| PUEUE_DEFAULT_GROUP.to_string())
}

// This is a helper function for easy retrieval of the current daemon state.
// The current daemon state is often needed in more complex commands.
pub async fn get_state(client: &mut Client) -> Result<State> {
    // Create the message payload and send it to the daemon.
    client.send_request(Request::Status).await?;

    // Check if we can receive the response from the daemon
    let response = client.receive_response().await?;

    match response {
        Response::Status(state) => Ok(*state),
        _ => unreachable!(),
    }
}

// This is a helper function for easy retrieval of a single task from the daemon state.
pub async fn get_task(client: &mut Client, task_id: usize) -> Result<Option<Task>> {
    // Create the message payload and send it to the daemon.
    client.send_request(Request::Status).await?;

    // Check if we can receive the response from the daemon
    let response = client.receive_response().await?;

    let state = match response {
        Response::Status(state) => state,
        _ => unreachable!(),
    };

    Ok(state.tasks.get(&task_id).cloned())
}

/// Most returned messages can be handled in a generic fashion.
/// However, some commands require to continuously receive messages (streaming).
///
/// If this function returns `Ok(true)`, the parent function will continue to receive
/// and handle messages from the daemon. Otherwise the client will simply exit.
fn handle_response(style: &OutputStyle, response: Response) -> Result<()> {
    match response {
        Response::Success(text) => print_success(style, &text),
        Response::Failure(text) => {
            print_error(style, &text);
            std::process::exit(1);
        }
        Response::Close => return Ok(()),
        _ => error!("Received unhandled response message"),
    };

    Ok(())
}

/// Handle any command.
///
/// This is the core entry point of the pueue client.
/// Based on the subcommand, the respective function in the [`super::commands`] module is
/// called.
pub async fn handle_command(client: &mut Client, subcommand: SubCommand) -> Result<()> {
    trace!(message = "Handling command", subcommand = ?subcommand);

    match subcommand {
        SubCommand::Add {
            command,
            working_directory,
            escape,
            start_immediately,
            stashed,
            group,
            delay_until,
            dependencies,
            priority,
            label,
            print_task_id,
            follow,
        } => {
            add_task(
                client,
                command,
                working_directory,
                escape,
                start_immediately,
                stashed,
                group,
                delay_until,
                dependencies,
                priority,
                label,
                print_task_id,
                follow,
            )
            .await
        }
        SubCommand::Clean {
            successful_only,
            group,
        } => clean(client, group, successful_only).await,
        SubCommand::Edit { task_ids } => edit(client, task_ids).await,
        SubCommand::Enqueue {
            task_ids,
            group,
            all,
            delay_until,
        } => enqueue(client, task_ids, group, all, delay_until).await,
        SubCommand::Env { cmd } => env(client, cmd).await,
        SubCommand::Follow { task_id, lines } => follow(client, task_id, lines).await,
        SubCommand::FormatStatus { group } => format_state(client, group).await,
        SubCommand::Group { cmd, json } => group(client, cmd, json).await,
        SubCommand::Kill {
            task_ids,
            group,
            all,
            signal,
        } => kill(client, task_ids, group, all, signal).await,
        SubCommand::Log {
            task_ids,
            group,
            all,
            json,
            lines,
            full,
        } => print_logs(client, task_ids, group, all, json, lines, full).await,
        SubCommand::Parallel {
            parallel_tasks,
            group,
        } => parallel(client, parallel_tasks, group).await,
        SubCommand::Pause {
            task_ids,
            group,
            all,
            wait,
        } => pause(client, task_ids, group, all, wait).await,
        SubCommand::Remove { task_ids } => remove(client, task_ids).await,
        SubCommand::Reset { force, groups } => reset(client, force, groups).await,
        SubCommand::Restart {
            task_ids,
            all_failed,
            failed_in_group,
            start_immediately,
            stashed,
            in_place,
            not_in_place,
            edit,
        } => {
            restart(
                client,
                task_ids,
                all_failed,
                failed_in_group,
                start_immediately,
                stashed,
                in_place,
                not_in_place,
                edit,
            )
            .await
        }
        SubCommand::Send { task_id, input } => send(client, task_id, input).await,
        SubCommand::Shutdown => shutdown(client).await,
        SubCommand::Stash {
            task_ids,
            group,
            all,
            delay_until,
        } => stash(client, task_ids, group, all, delay_until).await,
        SubCommand::Start {
            task_ids,
            group,
            all,
        } => start(client, task_ids, group, all).await,
        SubCommand::Status { query, json, group } => state(client, query, json, group).await,
        SubCommand::Switch {
            task_id_1,
            task_id_2,
        } => switch(client, task_id_1, task_id_2).await,
        SubCommand::Wait {
            task_ids,
            group,
            all,
            quiet,
            status,
        } => wait(client, task_ids, group, all, quiet, status).await,
        _ => bail!("unhandled WIP"),
    }
}

/// Prints a warning and prompt for a given action and tasks.
/// Returns `Ok(())` if the action was confirmed.
pub fn handle_user_confirmation(action: &str, task_ids: &[usize]) -> Result<()> {
    // printing warning and prompt
    let task_ids = task_ids
        .iter()
        .map(|t| format!("task{t}"))
        .collect::<Vec<String>>()
        .join(", ");
    eprintln!("You are trying to {action}: {task_ids}",);

    let mut input = String::new();

    loop {
        print!("Do you want to continue [Y/n]: ");
        stdout().flush()?;
        input.clear();
        stdin().read_line(&mut input)?;

        match input.chars().next().unwrap() {
            'N' | 'n' => {
                eprintln!("Aborted!");
                std::process::exit(1);
            }
            '\n' | 'Y' | 'y' => {
                break;
            }
            _ => {
                continue;
            }
        }
    }

    Ok(())
}
