use std::fmt::Display;
use std::sync::mpsc::Sender;

use pueue_lib::network::message::*;
use pueue_lib::network::protocol::socket_cleanup;
use pueue_lib::state::SharedState;

use crate::network::response_helper::*;

mod add;
mod clean;
mod edit;
mod enqueue;
mod group;
mod kill;
mod log;
mod parallel;
mod pause;
mod remove;
mod restart;
mod send;
mod start;
mod stash;
mod switch;

static SENDER_ERR: &str = "Failed to send message to task handler thread";

pub fn handle_message(message: Message, sender: &Sender<Message>, state: &SharedState) -> Message {
    match message {
        Message::Add(message) => add::add_task(message, sender, state),
        Message::Clean(message) => clean::clean(message, state),
        Message::Edit(message) => edit::edit(message, state),
        Message::EditRequest(task_id) => edit::edit_request(task_id, state),
        Message::Enqueue(message) => enqueue::enqueue(message, state),
        Message::Group(message) => group::group(message, state),
        Message::Kill(message) => kill::kill(message, sender, state),
        Message::Log(message) => log::get_log(message, state),
        Message::Parallel(message) => parallel::set_parallel_tasks(message, state),
        Message::Pause(message) => pause::pause(message, sender, state),
        Message::Remove(task_ids) => remove::remove(task_ids, state),
        Message::Reset(message) => reset(message, sender),
        Message::Restart(message) => restart::restart_multiple(message, sender, state),
        Message::Send(message) => send::send(message, sender, state),
        Message::Start(message) => start::start(message, sender, state),
        Message::Stash(task_ids) => stash::stash(task_ids, state),
        Message::Switch(message) => switch::switch(message, state),
        Message::Status => get_status(state),
        Message::DaemonShutdown => shutdown(sender, state),
        _ => create_failure_message("Not implemented yet"),
    }
}

/// Invoked when calling `pueue reset`.
/// Forward the reset request to the task handler.
/// The handler then kills all children and clears the task queue.
fn reset(message: ResetMessage, sender: &Sender<Message>) -> Message {
    sender.send(Message::Reset(message)).expect(SENDER_ERR);
    create_success_message("Everything is being reset right now.")
}

/// Invoked when calling `pueue status`.
/// Return the current state.
fn get_status(state: &SharedState) -> Message {
    let state = state.lock().unwrap().clone();
    Message::StatusResponse(Box::new(state))
}

/// Initialize the shutdown procedure.
/// At first, the unix socket will be removed.
///
/// Next, the DaemonShutdown Message will be forwarded to the TaskHandler.
/// The TaskHandler then gracefully shuts down all child processes
/// and exits with std::proces::exit(0).
fn shutdown(sender: &Sender<Message>, state: &SharedState) -> Message {
    // Do some socket cleanup (unix socket).
    {
        let state = state.lock().unwrap();
        if let Err(error) = socket_cleanup(&state.settings.shared) {
            println!("Failed to cleanup socket after shutdown.");
            println!("{}", error);
        };
    }

    // Notify the task handler.
    sender.send(Message::DaemonShutdown).expect(SENDER_ERR);

    create_success_message("Daemon is shutting down")
}

fn ok_or_failure_message<T, E: Display>(result: Result<T, E>) -> Result<T, Message> {
    match result {
        Ok(inner) => Ok(inner),
        Err(error) => Err(create_failure_message(format!(
            "Failed to save state. This is a bug: {}",
            error
        ))),
    }
}

#[macro_export]
macro_rules! ok_or_return_failure_message {
    ($expression:expr) => {
        match ok_or_failure_message($expression) {
            Ok(task_id) => task_id,
            Err(error) => return error,
        }
    };
}

#[cfg(test)]
mod fixtures {
    use std::collections::HashMap;
    use std::path::PathBuf;
    pub use std::sync::mpsc::Sender;
    use std::sync::{Arc, Mutex};

    pub use pueue_lib::network::message::*;
    pub use pueue_lib::network::protocol::socket_cleanup;
    pub use pueue_lib::settings::Settings;
    pub use pueue_lib::state::{SharedState, State};
    pub use pueue_lib::task::TaskResult;

    pub use pueue_lib::task::{Task, TaskStatus};

    pub use super::*;
    pub use crate::network::response_helper::*;

    pub fn get_settings(pueue_dir: PathBuf) -> Settings {
        let mut settings: Settings = Settings::default_config()
            .expect("Failed to get default config")
            .try_into()
            .expect("Failed to get test settings");
        settings.shared.pueue_directory = pueue_dir;

        settings
    }

    pub fn get_state(pueue_dir: PathBuf) -> SharedState {
        let settings = get_settings(pueue_dir.clone());

        // Create the normal pueue directories.
        let log_dir = pueue_dir.join("log");
        if !log_dir.exists() {
            std::fs::create_dir(log_dir).expect("Failed to create test log dir");
        }
        let task_log_dir = pueue_dir.join("task_log");
        if !task_log_dir.exists() {
            std::fs::create_dir(task_log_dir).expect("Failed to create test task log dir");
        }

        let state = State::new(&settings, None);
        Arc::new(Mutex::new(state))
    }

    /// Create a new task with stub data
    pub fn get_stub_task(id: &str, status: TaskStatus) -> Task {
        Task::new(
            format!("{}", id),
            "/tmp".to_string(),
            HashMap::new(),
            "default".to_string(),
            status,
            None,
            Vec::new(),
            None,
        )
    }

    pub fn get_stub_state(pueue_dir: PathBuf) -> SharedState {
        let state = get_state(pueue_dir);
        {
            // Queued task
            let mut state = state.lock().unwrap();
            let task = get_stub_task("0", TaskStatus::Queued);
            state.add_task(task);

            // Finished task
            let mut task = get_stub_task("1", TaskStatus::Done);
            task.result = Some(TaskResult::Success);
            state.add_task(task);

            // Stashed task
            let task = get_stub_task("2", TaskStatus::Stashed);
            state.add_task(task);

            // Running task
            let task = get_stub_task("3", TaskStatus::Running);
            state.add_task(task);

            // Paused task
            let task = get_stub_task("4", TaskStatus::Paused);
            state.add_task(task);
        }

        state
    }
}
