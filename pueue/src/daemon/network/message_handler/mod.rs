use std::fmt::Display;

use chrono::{DateTime, Local};
use pueue_lib::failure_msg;
use pueue_lib::network::message::*;
use pueue_lib::settings::Settings;
use pueue_lib::state::SharedState;

use crate::daemon::network::response_helper::*;

mod add;
mod clean;
mod edit;
mod enqueue;
mod env;
mod group;
mod kill;
mod log;
mod parallel;
mod pause;
mod remove;
mod reset;
mod restart;
mod send;
mod start;
mod stash;
mod switch;

pub use log::follow_log;

pub fn handle_message(message: Message, state: &SharedState, settings: &Settings) -> Message {
    match message {
        Message::Add(message) => add::add_task(settings, state, message),
        Message::Clean(message) => clean::clean(settings, state, message),
        Message::Edit(editable_tasks) => edit::edit(settings, state, editable_tasks),
        Message::EditRequest(task_ids) => edit::edit_request(state, task_ids),
        Message::EditRestore(task_ids) => edit::edit_restore(state, task_ids),
        Message::Env(message) => env::env(settings, state, message),
        Message::Enqueue(message) => enqueue::enqueue(settings, state, message),
        Message::Group(message) => group::group(settings, state, message),
        Message::Kill(message) => kill::kill(settings, state, message),
        Message::Log(message) => log::get_log(settings, state, message),
        Message::Parallel(message) => parallel::set_parallel_tasks(message, state),
        Message::Pause(message) => pause::pause(settings, state, message),
        Message::Remove(task_ids) => remove::remove(settings, state, task_ids),
        Message::Reset(message) => reset::reset(settings, state, message),
        Message::Restart(message) => restart::restart_multiple(settings, state, message),
        Message::Send(message) => send::send(state, message),
        Message::Start(message) => start::start(settings, state, message),
        Message::Stash(message) => stash::stash(settings, state, message),
        Message::Switch(message) => switch::switch(settings, state, message),
        Message::Status => get_status(state),
        _ => create_failure_message("Not yet implemented"),
    }
}

/// Invoked when calling `pueue status`.
/// Return the current state.
fn get_status(state: &SharedState) -> Message {
    let state = state.lock().unwrap().clone();
    Message::StatusResponse(Box::new(state))
}

// If the enqueue at time is today, only show the time. Otherwise, include the date.
fn format_datetime(settings: &Settings, enqueue_at: &DateTime<Local>) -> String {
    let format_string = if enqueue_at.date_naive() == Local::now().date_naive() {
        &settings.client.status_time_format
    } else {
        &settings.client.status_datetime_format
    };
    enqueue_at.format(format_string).to_string()
}

fn ok_or_failure_message<T, E: Display>(result: Result<T, E>) -> Result<T, Message> {
    match result {
        Ok(inner) => Ok(inner),
        Err(error) => Err(failure_msg!("Failed to save state. This is a bug: {error}")),
    }
}

#[macro_export]
macro_rules! ok_or_save_state_failure {
    ($expression:expr) => {
        match ok_or_failure_message($expression) {
            Ok(task_id) => task_id,
            Err(error) => return error,
        }
    };
}

#[cfg(test)]
mod fixtures {
    use chrono::{DateTime, Duration, Local};
    use std::collections::HashMap;
    use std::env::temp_dir;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    pub use pueue_lib::settings::Settings;
    pub use pueue_lib::state::{SharedState, State, PUEUE_DEFAULT_GROUP};
    pub use pueue_lib::task::{Task, TaskResult, TaskStatus};

    // A simple helper struct to keep the boilerplate for TaskStatus creation down.
    pub enum StubStatus {
        Queued,
        Running,
        Paused,
        Stashed { enqueue_at: Option<DateTime<Local>> },
        Done(TaskResult),
    }

    pub fn get_settings() -> (Settings, TempDir) {
        let tempdir = TempDir::new().expect("Failed to create test pueue directory");
        let mut settings = Settings::default();
        settings.shared.pueue_directory = Some(tempdir.path().to_owned());

        (settings, tempdir)
    }

    pub fn get_state() -> (SharedState, Settings, TempDir) {
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

        let state = State::new();
        (Arc::new(Mutex::new(state)), settings, tempdir)
    }

    /// Create a new task with stub data in the given group
    pub fn get_stub_task_in_group(id: &str, group: &str, status: StubStatus) -> Task {
        // Build a proper Task status based on the simplified requested stub status.
        let enqueued_at = Local::now() - Duration::minutes(5);
        let start = Local::now() - Duration::minutes(4);
        let end = Local::now() - Duration::minutes(1);

        let status = match status {
            StubStatus::Stashed { enqueue_at } => TaskStatus::Stashed { enqueue_at },
            StubStatus::Queued => TaskStatus::Queued { enqueued_at },
            StubStatus::Running => TaskStatus::Running { enqueued_at, start },
            StubStatus::Paused => TaskStatus::Paused { enqueued_at, start },
            StubStatus::Done(result) => TaskStatus::Done {
                enqueued_at,
                start,
                end,
                result,
            },
        };

        Task::new(
            id.to_string(),
            temp_dir(),
            HashMap::new(),
            group.to_string(),
            status,
            Vec::new(),
            0,
            None,
        )
    }

    /// Create a new task with stub data
    pub fn get_stub_task(id: &str, status: StubStatus) -> Task {
        get_stub_task_in_group(id, PUEUE_DEFAULT_GROUP, status)
    }

    pub fn get_stub_state() -> (SharedState, Settings, TempDir) {
        let (state, settings, tempdir) = get_state();
        {
            // Queued task
            let mut state = state.lock().unwrap();
            let task = get_stub_task("0", StubStatus::Queued);
            state.add_task(task);

            // Finished task
            let task = get_stub_task("1", StubStatus::Done(TaskResult::Success));
            state.add_task(task);

            // Stashed task
            let task = get_stub_task("2", StubStatus::Stashed { enqueue_at: None });
            state.add_task(task);

            // Running task
            let task = get_stub_task("3", StubStatus::Running);
            state.add_task(task);

            // Paused task
            let task = get_stub_task("4", StubStatus::Paused);
            state.add_task(task);
        }

        (state, settings, tempdir)
    }
}
