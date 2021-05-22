use std::path::PathBuf;
use std::process::Child;
use std::process::Stdio;
use std::sync::mpsc::Receiver;

use std::{
    collections::{BTreeMap, HashMap},
    sync::MutexGuard,
};

use anyhow::{Context, Result};
use chrono::prelude::*;
use handlebars::Handlebars;
use log::{debug, error, info};

use pueue_lib::log::*;
use pueue_lib::network::message::*;
use pueue_lib::state::{GroupStatus, SharedState, State};
use pueue_lib::task::{Task, TaskResult, TaskStatus};

use crate::platform::process_helper::*;

mod callback;
/// Logic for handling dependencies
mod dependencies;
/// Logic for finishing and cleaning up completed tasks.
mod finish_task;
/// This module contains all logic that's triggered by messages received via the mpsc channel.
/// These messages are sent by the threads that handle the client messages.
mod messages;
/// Everything regarding actually spawning task processes.
mod spawn_task;

type LockedState<'a> = MutexGuard<'a, State>;

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
                    "Initializing graceful shutdown. Encountered error in TaskManager: {}",
                    err
                );
                $task_manager.emergency_shutdown = true;
                return;
            }
            Ok(inner) => inner,
        }
    };
}

pub struct TaskHandler {
    state: SharedState,
    receiver: Receiver<Message>,
    children: BTreeMap<usize, Child>,
    callbacks: Vec<Child>,
    full_reset: bool,
    graceful_shutdown: bool,
    emergency_shutdown: bool,
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
    pub fn new(state: SharedState, receiver: Receiver<Message>) -> Self {
        // Extract some static settings we often need.
        // This prevents locking the State all the time.
        let (pueue_directory, callback, callback_log_lines) = {
            let state = state.lock().unwrap();
            (
                state.settings.shared.pueue_directory(),
                state.settings.daemon.callback.clone(),
                state.settings.daemon.callback_log_lines,
            )
        };

        TaskHandler {
            state,
            receiver,
            children: BTreeMap::new(),
            callbacks: Vec::new(),
            full_reset: false,
            graceful_shutdown: false,
            emergency_shutdown: false,
            pueue_directory,
            callback,
            callback_log_lines,
        }
    }

    /// Main loop of the task handler.
    /// In here a few things happen:
    ///
    /// - Receive and handle instructions from the client.
    /// - Handle finished tasks, i.e. cleanup processes, update statuses.
    /// - If the client requested a reset: reset the state if all children have been killed and handled.
    /// - Callback handling logic. This is rather uncritical.
    /// - Enqueue any stashed processes which are ready for being queued.
    /// - Ensure tasks with dependencies have no failed ancestors
    /// - Check whether we can spawn new tasks.
    pub fn run(&mut self) {
        loop {
            self.handle_emergency_shutdown();
            self.receive_messages();
            self.handle_finished_tasks();
            self.handle_reset();
            self.check_callbacks();
            self.enqueue_delayed_tasks();
            self.check_failed_dependencies();

            // Don't start new tasks, if we're in the middle of a reset or shutdown.
            if !self.full_reset && !self.graceful_shutdown && !self.emergency_shutdown {
                self.spawn_new();
            }
        }
    }

    /// We encountered an error and the daemon needs to gracefully shutdown.
    /// Initiate a full reset.
    fn handle_emergency_shutdown(&mut self) {
        if self.emergency_shutdown && !self.full_reset {
            self.reset(false);
        }
    }

    /// Users can issue to reset the daemon.
    /// If that's the case, the `self.full_reset` flag is set to true, all children are killed
    /// and no new tasks will be spawned.
    /// This function checks, if all killed children have been handled.
    /// If that's the case, completely reset the state
    fn handle_reset(&mut self) {
        // The daemon got a reset request and all children already finished
        if self.full_reset && self.children.is_empty() {
            let mut state = self.state.lock().unwrap();
            if let Err(error) = state.reset() {
                error!("Failed to reset state with error: {:?}", error);
            };
            state.set_status_for_all_groups(GroupStatus::Running);

            if let Err(error) = reset_task_log_directory(&self.pueue_directory) {
                panic!("Error while resetting task log directory: {}", error);
            };
            self.full_reset = false;

            // Actually exit the program in case we're supposed to.
            // Depending on the current shutdown type, we exit with different exit codes.
            if self.graceful_shutdown {
                if let Err(error) = crate::pid::cleanup_pid_file(&self.pueue_directory) {
                    println!("Failed to cleanup pid after shutdown.");
                    println!("{}", error);
                }

                std::process::exit(0);
            } else if self.emergency_shutdown {
                if let Err(error) = crate::pid::cleanup_pid_file(&self.pueue_directory) {
                    println!("Failed to cleanup pid after shutdown.");
                    println!("{}", error);
                }

                std::process::exit(1);
            }
        }
    }

    /// Kill all children by using the `kill` function.
    /// Set the respective group's statuses to `Reset`. This will prevent new tasks from being spawned.
    fn reset(&mut self, kill_children: bool) {
        {
            let mut state = self.state.lock().unwrap();
            state.set_status_for_all_groups(GroupStatus::Paused);
        }

        self.full_reset = true;
        self.kill(vec![], String::new(), true, kill_children, None);
    }

    /// As time passes, some delayed tasks may need to be enqueued.
    /// Gather all stashed tasks and enqueue them if it is after the task's enqueue_at
    fn enqueue_delayed_tasks(&mut self) {
        let mut state = self.state.lock().unwrap();

        let mut changed = false;
        for (_, task) in state.tasks.iter_mut() {
            if task.status != TaskStatus::Stashed {
                continue;
            }

            if let Some(time) = task.enqueue_at {
                if time <= Local::now() {
                    info!("Enqueuing delayed task : {}", task.id);

                    task.status = TaskStatus::Queued;
                    task.enqueue_at = None;
                    changed = true;
                }
            }
        }
        // Save the state if a task has been enqueued
        if changed {
            ok_or_shutdown!(self, state.save());
        }
    }

    /// This is a small wrapper around the real platform dependant process handling logic
    /// It only ensures, that the process we want to manipulate really does exists.
    fn perform_action(&mut self, id: usize, action: ProcessAction, children: bool) -> Result<bool> {
        match self.children.get(&id) {
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
