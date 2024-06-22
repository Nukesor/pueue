use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::process::Child;

use anyhow::Result;
use chrono::prelude::*;
use handlebars::Handlebars;
use log::{debug, error, info};

use pueue_lib::children::Children;
use pueue_lib::log::*;
use pueue_lib::network::message::*;
use pueue_lib::network::protocol::socket_cleanup;
use pueue_lib::process_helper::*;
use pueue_lib::settings::Settings;
use pueue_lib::state::{GroupStatus, SharedState};
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::daemon::pid::cleanup_pid_file;
use crate::daemon::state_helper::{reset_state, save_state};

use super::process_handler::finish::handle_finished_tasks;
use super::process_handler::spawn::spawn_new;
use super::state_helper::LockedState;

mod callback;
/// Logic for handling dependencies
mod dependencies;

/// This is a little helper macro, which looks at a critical result and shuts the
/// TaskHandler down, if an error occurred. This is mostly used if the state cannot
/// be written due to IO errors.
/// Those errors are considered unrecoverable and we should initiate a graceful shutdown
/// immediately.
#[macro_export]
macro_rules! ok_or_shutdown {
    ($settings:expr, $state:expr, $result:expr) => {
        match $result {
            Err(err) => {
                use pueue_lib::network::message::Shutdown;
                use $crate::daemon::process_handler::initiate_shutdown;
                error!("Initializing graceful shutdown. Encountered error in TaskHandler: {err}");
                initiate_shutdown($settings, $state, Shutdown::Emergency);
                return;
            }
            Ok(inner) => inner,
        }
    };
}

pub struct TaskHandler {
    /// The state that's shared between the TaskHandler and the message handling logic.
    state: SharedState,
    /// These are the currently running callbacks. They're usually very short-lived.
    callbacks: Vec<Child>,
    /// The settings that are passed at program start.
    settings: Settings,

    // Some static settings that are extracted from `settings` for convenience purposes.
    pueue_directory: PathBuf,
}

impl TaskHandler {
    pub fn new(shared_state: SharedState, settings: Settings) -> Self {
        // Clone the pointer, as we need to regularly access it inside the TaskHandler.
        let state_clone = shared_state.clone();
        let mut state = state_clone.lock().unwrap();

        // Initialize the subprocess management structure.
        let mut pools = BTreeMap::new();
        for group in state.groups.keys() {
            pools.insert(group.clone(), BTreeMap::new());
        }
        state.children = Children(pools);

        TaskHandler {
            state: shared_state,
            callbacks: Vec::new(),
            pueue_directory: settings.shared.pueue_directory(),
            settings,
        }
    }

    /// Main loop of the task handler.
    /// In here a few things happen:
    ///
    /// - Receive and handle instructions from the client.
    /// - Handle finished tasks, i.e. cleanup processes, update statuses.
    /// - Callback handling logic. This is rather uncritical.
    /// - Enqueue any stashed processes which are ready for being queued.
    /// - Ensure tasks with dependencies have no failed ancestors
    /// - Whether whe should perform a shutdown.
    /// - If the client requested a reset: reset the state if all children have been killed and handled.
    /// - Check whether we can spawn new tasks.
    ///
    /// This first step waits for 200ms while receiving new messages.
    /// This prevents this loop from running hot, but also means that we only check if a new task
    /// can be scheduled or if tasks are finished, every 200ms.
    pub fn run(&mut self) {
        loop {
            {
                let state_clone = self.state.clone();
                let mut state = state_clone.lock().unwrap();

                handle_finished_tasks(&self.settings, &mut state);
                enqueue_delayed_tasks(&self.settings, &mut state);
                self.check_failed_dependencies(&mut state);

                if state.shutdown.is_some() {
                    // Check if we're in shutdown.
                    // If all tasks are killed, we do some cleanup and exit.
                    handle_shutdown(&self.settings, &mut state);
                } else if state.full_reset {
                    // Wait until all tasks are killed.
                    // Once they are, reset everything and go back to normal
                    handle_reset(&self.settings, &mut state);
                } else {
                    // Only start new tasks, if we aren't in the middle of a reset or shutdown.
                    spawn_new(&self.settings, &mut state);
                }
            }

            self.check_callbacks();
        }
    }
}

/// Check if all tasks are killed.
/// If they aren't, we'll wait a little longer.
/// Once they're, we do some cleanup and exit.
fn handle_shutdown(settings: &Settings, state: &mut LockedState) {
    // There are still active tasks. Continue waiting until they're killed and cleaned up.
    if state.children.has_active_tasks() {
        return;
    }

    // Remove the unix socket.
    if let Err(error) = socket_cleanup(&settings.shared) {
        println!("Failed to cleanup socket during shutdown.");
        println!("{error}");
    }

    // Cleanup the pid file
    if let Err(error) = cleanup_pid_file(&settings.shared.pid_path()) {
        println!("Failed to cleanup pid during shutdown.");
        println!("{error}");
    }

    // Actually exit the program the way we're supposed to.
    // Depending on the current shutdown type, we exit with different exit codes.
    if matches!(state.shutdown, Some(Shutdown::Emergency)) {
        std::process::exit(1);
    }
    std::process::exit(0);
}

/// Users can issue to reset the daemon.
/// If that's the case, the `self.full_reset` flag is set to true, all children are killed
/// and no new tasks will be spawned.
/// This function checks, if all killed children have been handled.
/// If that's the case, completely reset the state
fn handle_reset(settings: &Settings, state: &mut LockedState) {
    // Don't do any reset logic, if we aren't in reset mode or if some children are still up.
    if state.children.has_active_tasks() {
        return;
    }

    if let Err(error) = reset_state(state, settings) {
        error!("Failed to reset state with error: {error:?}");
    };

    if let Err(error) = reset_task_log_directory(&settings.shared.pueue_directory()) {
        panic!("Error while resetting task log directory: {error}");
    };
    state.full_reset = false;
}

/// As time passes, some delayed tasks may need to be enqueued.
/// Gather all stashed tasks and enqueue them if it is after the task's enqueue_at
fn enqueue_delayed_tasks(settings: &Settings, state: &mut LockedState) {
    let mut changed = false;
    for (_, task) in state.tasks.iter_mut() {
        if let TaskStatus::Stashed {
            enqueue_at: Some(time),
        } = task.status
        {
            if time <= Local::now() {
                info!("Enqueuing delayed task : {}", task.id);

                task.status = TaskStatus::Queued;
                task.enqueued_at = Some(Local::now());
                changed = true;
            }
        }
    }
    // Save the state if a task has been enqueued
    if changed {
        ok_or_shutdown!(settings, state, save_state(state, settings));
    }
}
