//! This module contains the logic for all non-trivial commands, such as `follow`, `restart`,
//! `wait`, etc.
//!
//! "non-trivial" vaguely means that we, for instance, have to do additional requests to the
//! daemon, open some files on the filesystem, edit files and so on.
//! All commands that cannot be simply handled by handling requests or using `pueue_lib`.

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

pub use add::add_task;
pub use clean::clean;
pub use edit::edit;
pub use enqueue::enqueue;
pub use env::env;
pub use follow::follow;
pub use group::group;
pub use kill::kill;
pub use log::print_logs;
pub use parallel::parallel;
pub use pause::pause;
pub use remove::remove;
pub use reset::reset;
pub use restart::restart;
pub use send::send;
pub use shutdown::shutdown;
pub use start::start;
pub use stash::stash;
pub use state::{format_state, state};
pub use switch::switch;
pub use wait::{wait, WaitTargetStatus};

use super::{
    client::Client,
    display::{print_error, print_success, OutputStyle},
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
