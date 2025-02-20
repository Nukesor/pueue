use std::fmt::Display;

use pueue_lib::{
    Settings, failure_msg,
    network::{message::*, protocol::send_response, socket::GenericStream},
};

use crate::{
    daemon::{internal_state::SharedState, process_handler::initiate_shutdown},
    internal_prelude::*,
};

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

pub async fn handle_request(
    stream: &mut GenericStream,
    request: Request,
    state: &SharedState,
    settings: &Settings,
) -> Result<()> {
    let response = match request {
        // The client requested the output of a task.
        // Since this involves streaming content, we have to do some special handling.
        Request::Stream(payload) => {
            let pueue_directory = settings.shared.pueue_directory();
            follow_log(&pueue_directory, stream, state, payload).await?
        }
        // To initiated a shutdown, a flag in Pueue's state is set that informs the TaskHandler
        // to perform a graceful shutdown.
        //
        // However, this is an edge-case as we have respond to the client first.
        // Otherwise it might happen, that the daemon shuts down too fast and we aren't
        // capable of actually sending the message back to the client.
        Request::DaemonShutdown(shutdown_type) => {
            let response = create_success_response("Daemon is shutting down");
            send_response(response, stream).await?;

            let mut state = state.lock().unwrap();
            initiate_shutdown(settings, &mut state, shutdown_type);

            return Ok(());
        }
        Request::Add(message) => add::add_task(settings, state, message),
        Request::Clean(message) => clean::clean(settings, state, message),
        Request::EditedTasks(editable_tasks) => edit::edit(settings, state, editable_tasks),
        Request::EditRequest(task_ids) => edit::edit_request(state, task_ids),
        Request::EditRestore(task_ids) => edit::edit_restore(state, task_ids),
        Request::Env(message) => env::env(settings, state, message),
        Request::Enqueue(message) => enqueue::enqueue(settings, state, message),
        Request::Group(message) => group::group(settings, state, message),
        Request::Kill(message) => kill::kill(settings, state, message),
        Request::Log(message) => log::get_log(settings, state, message),
        Request::Parallel(message) => parallel::set_parallel_tasks(message, state),
        Request::Pause(message) => pause::pause(settings, state, message),
        Request::Remove(task_ids) => remove::remove(settings, state, task_ids),
        Request::Reset(message) => reset::reset(settings, state, message),
        Request::Restart(message) => restart::restart_multiple(settings, state, message),
        Request::Send(message) => send::send(state, message),
        Request::Start(message) => start::start(settings, state, message),
        Request::Stash(message) => stash::stash(settings, state, message),
        Request::Switch(message) => switch::switch(settings, state, message),
        Request::Status => get_status(state),
    };

    send_response(response, stream).await?;

    Ok(())
}

/// Invoked when calling `pueue status`.
/// Return the current state.
fn get_status(state: &SharedState) -> Response {
    let state = state.lock().unwrap().clone();
    Response::Status(Box::new(state.inner))
}

fn ok_or_failure_message<T, E: Display>(result: Result<T, E>) -> Result<T, Response> {
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
    use std::{
        collections::HashMap,
        env::temp_dir,
        sync::{Arc, Mutex},
    };

    use chrono::{DateTime, Duration, Local};
    pub use pueue_lib::{
        settings::Settings,
        state::PUEUE_DEFAULT_GROUP,
        task::{Task, TaskResult, TaskStatus},
    };
    use tempfile::TempDir;

    use crate::daemon::internal_state::{SharedState, state::InternalState};

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

        let state = InternalState::new();
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
