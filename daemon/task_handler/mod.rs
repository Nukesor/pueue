use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::process::Child;
use std::process::Stdio;

use anyhow::Result;
use chrono::prelude::*;
use crossbeam_channel::Receiver;
use handlebars::Handlebars;
use log::{debug, error, info};

use pueue_lib::log::*;
use pueue_lib::network::message::*;
use pueue_lib::network::protocol::socket_cleanup;
use pueue_lib::state::{GroupStatus, SharedState};
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::pid::cleanup_pid_file;
use crate::platform::process_helper::*;
use crate::state_helper::{reset_state, save_state};

mod callback;
/// A helper newtype struct, which implements convenience methods for our child process management
/// datastructure.
mod children;
/// Logic for handling dependencies
mod dependencies;
/// Logic for finishing and cleaning up completed tasks.
mod finish_task;
/// This module contains all logic that's triggered by messages received via the mpsc channel.
/// These messages are sent by the threads that handle the client messages.
mod messages;
/// Everything regarding actually spawning task processes.
mod spawn_task;

use children::Children;

/// This is a little helper macro, which looks at a critical result and shuts the
/// TaskHandler down, if an error occurred. This is mostly used if the state cannot.
/// be written due to IO errors.
/// Those errors are considered unrecoverable and we should initiate a graceful shutdown
/// immediately.
#[macro_export]
macro_rules! ok_or_shutdown {
    ($task_manager:ident, $result:expr) => {
        match $result {
            Err(err) => {
                error!(
                    "Initializing graceful shutdown. Encountered error in TaskHandler: {}",
                    err
                );
                $task_manager.initiate_shutdown(Shutdown::Emergency);
                return;
            }
            Ok(inner) => inner,
        }
    };
}

pub struct TaskHandler {
    /// The state that's shared between the TaskHandler and the message handling logic.
    state: SharedState,
    /// The receiver for the MPSC channel that's used to push notificatoins from our message
    /// handling to the TaskHandler.
    receiver: Receiver<Message>,
    /// A mapping with this structure:
    /// BTreeMap<group, BTreeMap<group_worker_id, (task_id, Subprocess handle)>
    children: Children,
    /// These are the currently running callbacks. They're usually very short-lived.
    callbacks: Vec<Child>,
    /// A simple flag which is used to signal that we're currently doing a full reset of the daemon.
    /// This flag prevents new tasks from being spawned.
    full_reset: bool,
    /// Whether we're currently in the process of a graceful shutdown.
    /// Depending on the shutdown type, we're exiting with different exitcodes.
    shutdown: Option<Shutdown>,

    // Some static settings that are extracted from `state.settings` for convenience purposes.
    pueue_directory: PathBuf,
    callback: Option<String>,
    callback_log_lines: usize,
}

/// Pueue directly interacts with processes.
/// Since these interactions can vary depending on the current platform, this enum is introduced.
/// The intend is to keep any platform specific code out of the top level code.
/// Even if that implicates adding some layers of abstraction.
#[derive(Debug)]
pub enum ProcessAction {
    Pause,
    Resume,
}

impl TaskHandler {
    pub fn new(shared_state: SharedState, receiver: Receiver<Message>) -> Self {
        // Clone the pointer, as we need to access it, but also put it into the TaskHandler.
        let state_clone = shared_state.clone();
        let state = state_clone.lock().unwrap();

        // Initialize the subprocess management structure.
        let mut pools = BTreeMap::new();
        for group in state.groups.keys() {
            pools.insert(group.clone(), BTreeMap::new());
        }

        TaskHandler {
            state: shared_state,
            receiver,
            children: Children(pools),
            callbacks: Vec::new(),
            full_reset: false,
            shutdown: None,
            pueue_directory: state.settings.shared.pueue_directory(),
            callback: state.settings.daemon.callback.clone(),
            callback_log_lines: state.settings.daemon.callback_log_lines,
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
    pub fn run(&mut self) {
        loop {
            self.receive_messages();
            self.handle_finished_tasks();
            self.check_callbacks();
            self.enqueue_delayed_tasks();
            self.check_failed_dependencies();

            if self.shutdown.is_some() {
                // Check if we're in shutdown.
                // If all tasks are killed, we do some cleanup and exit.
                self.handle_shutdown();
            } else if self.full_reset {
                // Wait until all tasks are killed.
                // Once they are, reset everything and go back to normal
                self.handle_reset();
            } else {
                // Only start new tasks, if we aren't in the middle of a reset or shutdown.
                self.spawn_new();
            }
        }
    }

    /// Initiate shutdown, which includes killing all children and pausing all groups.
    /// We don't have to pause any groups, as no new tasks will be spawned during shutdown anyway.
    /// Any groups with queued tasks, will be automatically paused on state-restoration.
    fn initiate_shutdown(&mut self, shutdown: Shutdown) {
        self.shutdown = Some(shutdown);

        self.kill(TaskSelection::All, false, false, None);
    }

    /// Check if all tasks are killed.
    /// If they aren't, we'll wait a little longer.
    /// Once they're, we do some cleanup and exit.
    fn handle_shutdown(&mut self) {
        // There are still active tasks. Continue waiting until they're killed and cleaned up.
        if self.children.has_active_tasks() {
            return;
        }

        // Lock the state. This prevents any further connections/alterations from this point on.
        let state = self.state.lock().unwrap();

        // Remove the unix socket.
        if let Err(error) = socket_cleanup(&state.settings.shared) {
            println!("Failed to cleanup socket during shutdown.");
            println!("{}", error);
        }

        // Cleanup the pid file
        if let Err(error) = cleanup_pid_file(&self.pueue_directory) {
            println!("Failed to cleanup pid during shutdown.");
            println!("{}", error);
        }

        // Actually exit the program the way we're supposed to.
        // Depending on the current shutdown type, we exit with different exit codes.
        if matches!(self.shutdown, Some(Shutdown::Emergency)) {
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    /// Users can issue to reset the daemon.
    /// If that's the case, the `self.full_reset` flag is set to true, all children are killed
    /// and no new tasks will be spawned.
    /// This function checks, if all killed children have been handled.
    /// If that's the case, completely reset the state
    fn handle_reset(&mut self) {
        // Don't do any reset logic, if we aren't in reset mode or if some children are still up.
        if self.children.has_active_tasks() {
            return;
        }

        let mut state = self.state.lock().unwrap();
        if let Err(error) = reset_state(&mut state) {
            error!("Failed to reset state with error: {:?}", error);
        };

        if let Err(error) = reset_task_log_directory(&self.pueue_directory) {
            panic!("Error while resetting task log directory: {}", error);
        };
        self.full_reset = false;
    }

    /// Kill all children by using the `kill` function.
    /// Set the respective group's statuses to `Reset`. This will prevent new tasks from being spawned.
    fn reset(&mut self, kill_children: bool) {
        self.full_reset = true;
        self.kill(TaskSelection::All, kill_children, false, None);
    }

    /// As time passes, some delayed tasks may need to be enqueued.
    /// Gather all stashed tasks and enqueue them if it is after the task's enqueue_at
    fn enqueue_delayed_tasks(&mut self) {
        let state_clone = self.state.clone();
        let mut state = state_clone.lock().unwrap();

        let mut changed = false;
        for (_, task) in state.tasks.iter_mut() {
            if let TaskStatus::Stashed {
                enqueue_at: Some(time),
            } = task.status
            {
                if time <= Local::now() {
                    info!("Enqueuing delayed task : {}", task.id);

                    task.status = TaskStatus::Queued;
                    changed = true;
                }
            }
        }
        // Save the state if a task has been enqueued
        if changed {
            ok_or_shutdown!(self, save_state(&state));
        }
    }

    /// This is a small wrapper around the real platform dependant process handling logic
    /// It only ensures, that the process we want to manipulate really does exists.
    fn perform_action(&mut self, id: usize, action: ProcessAction, children: bool) -> Result<bool> {
        match self.children.get_child(id) {
            Some(child) => {
                debug!("Executing action {:?} to {}", action, id);
                run_action_on_child(child, &action, children)?;

                Ok(true)
            }
            None => {
                error!(
                    "Tried to execute action {:?} to non existing task {}",
                    action, id
                );
                Ok(false)
            }
        }
    }
}
