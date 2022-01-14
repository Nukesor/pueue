use crossbeam_channel::Sender;
use std::fmt::Display;

use pueue_lib::network::message::*;
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

pub static SENDER_ERR: &str = "Failed to send message to task handler thread";

pub fn handle_message(message: Message, sender: &Sender<Message>, state: &SharedState) -> Message {
    match message {
        Message::Add(message) => add::add_task(message, sender, state),
        Message::Clean(message) => clean::clean(message, state),
        Message::Edit(message) => edit::edit(message, state),
        Message::EditRequest(task_id) => edit::edit_request(task_id, state),
        Message::EditRestore(task_id) => edit::edit_restore(task_id, state),
        Message::Enqueue(message) => enqueue::enqueue(message, state),
        Message::Group(message) => group::group(message, sender, state),
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
        _ => create_failure_message("Not yet implemented"),
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

fn ok_or_failure_message<T, E: Display>(result: Result<T, E>) -> Result<T, Message> {
    match result {
        Ok(inner) => Ok(inner),
        Err(error) => Err(create_failure_message(format!(
            "Failed to save state. This is a bug: {error}"
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
    pub use crossbeam_channel::Sender;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    pub use pueue_lib::network::message::*;
    pub use pueue_lib::network::protocol::socket_cleanup;
    pub use pueue_lib::settings::Settings;
    pub use pueue_lib::state::{SharedState, State, PUEUE_DEFAULT_GROUP};
    pub use pueue_lib::task::{Task, TaskResult, TaskStatus};

    pub use super::*;
    pub use crate::network::response_helper::*;

    pub fn get_settings() -> (Settings, TempDir) {
        let tempdir = TempDir::new().expect("Failed to create test pueue directory");
        let mut settings = Settings::default();
        settings.shared.pueue_directory = Some(tempdir.path().to_owned());

        (settings, tempdir)
    }

    pub fn get_state() -> (SharedState, TempDir) {
        let (settings, tempdir) = get_settings();

        // Create the normal pueue directories.
        let log_dir = tempdir.path().join("log");
        if !log_dir.exists() {
            std::fs::create_dir(log_dir).expect("Failed to create test log dir");
        }
        let task_log_dir = tempdir.path().join("task_log");
        if !task_log_dir.exists() {
            std::fs::create_dir(task_log_dir).expect("Failed to create test task log dir");
        }

        let state = State::new(&settings, None);
        (Arc::new(Mutex::new(state)), tempdir)
    }

    /// Create a new task with stub data in the given group
    pub fn get_stub_task_in_group(id: &str, group: &str, status: TaskStatus) -> Task {
        Task::new(
            id.to_string(),
            "/tmp".to_string(),
            HashMap::new(),
            group.to_string(),
            status,
            Vec::new(),
            None,
        )
    }

    /// Create a new task with stub data
    pub fn get_stub_task(id: &str, status: TaskStatus) -> Task {
        get_stub_task_in_group(id, PUEUE_DEFAULT_GROUP, status)
    }

    pub fn get_stub_state() -> (SharedState, TempDir) {
        let (state, tempdir) = get_state();
        {
            // Queued task
            let mut state = state.lock().unwrap();
            let task = get_stub_task("0", TaskStatus::Queued);
            state.add_task(task);

            // Finished task
            let task = get_stub_task("1", TaskStatus::Done(TaskResult::Success));
            state.add_task(task);

            // Stashed task
            let task = get_stub_task("2", TaskStatus::Stashed { enqueue_at: None });
            state.add_task(task);

            // Running task
            let task = get_stub_task("3", TaskStatus::Running);
            state.add_task(task);

            // Paused task
            let task = get_stub_task("4", TaskStatus::Paused);
            state.add_task(task);
        }

        (state, tempdir)
    }
}
