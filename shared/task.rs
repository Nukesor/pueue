use ::chrono::prelude::*;
use ::serde_derive::{Deserialize, Serialize};

use chrono::Duration;

use crate::state::State;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Waiting {
    /// Waiting for execution by the TaskManager
    Queued,

    /// Waiting to be enqueued manually
    Stashed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Running {
    /// The task has been started and is currently running
    Running,
    /// The task has been started, but manually paused afterwards
    /// Won't be started unless the user manually does so.
    Paused,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Finished {
    /// The task has successfully finished
    Done,
    /// The task somehow failed
    Failed(i32),
    /// The task has been actively killed
    Killed,

    UnableToSpawn(String),

    Errored,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskState {
    Waiting {
        status: Waiting,
        enqueue_at: Option<DateTime<Local>>,
        dependencies: Vec<usize>,
        locked: bool,
    },
    Running {
        status: Running,
        start: DateTime<Local>,
    },
    Finished {
        status: Finished,
        start: DateTime<Local>,
        end: DateTime<Local>,
        stdout: String,
        stderr: String,
    },
}

/// Representation of a task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: usize,
    pub command: String,
    pub path: String,
    state: TaskState,
}

impl Task {
    pub fn new(
        command: String,
        path: String,
        should_be_stashed: bool,
        dependencies: Vec<usize>,
    ) -> Task {
        Task {
            id: 0,
            command,
            path,
            state: TaskState::Waiting {
                status: if should_be_stashed {
                    Waiting::Stashed
                } else {
                    Waiting::Queued
                },
                enqueue_at: None,
                dependencies,
                locked: false,
            },
        }
    }

    pub fn from_task(task: &Task, should_be_stashed: bool) -> Task {
        Task::new(
            task.command.clone(),
            task.path.clone(),
            should_be_stashed,
            Vec::new(),
        )
    }

    pub fn state(&self) -> &TaskState {
        &self.state
    }
}

// Test if task meet requirements
impl Task {
    pub fn is_started(&self) -> bool {
        match self.state {
            TaskState::Running { .. } => true,
            _ => false,
        }
    }

    pub fn is_running(&self) -> bool {
        match self.state {
            TaskState::Running {
                status: Running::Running,
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_finished(&self) -> bool {
        match self.state {
            TaskState::Finished { .. } => true,
            _ => false,
        }
    }

    pub fn is_waiting(&self) -> bool {
        match self.state {
            TaskState::Waiting { .. } => true,
            _ => false,
        }
    }

    pub fn is_done(&self) -> bool {
        match self.state {
            TaskState::Finished {
                status: Finished::Done,
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_locked(&self) -> bool {
        match self.state {
            TaskState::Waiting { locked, .. } => locked,
            _ => false,
        }
    }

    pub fn is_errored(&self) -> bool {
        match self.state {
            TaskState::Finished {
                status: Finished::Failed(_),
                ..
            }
            | TaskState::Finished {
                status: Finished::Killed,
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_paused(&self) -> bool {
        match self.state {
            TaskState::Running {
                status: Running::Paused,
                ..
            } => true,
            _ => false,
        }
    }


    pub fn is_delayed(&self) -> bool {
        match self.state {
            TaskState::Waiting {
                enqueue_at: Some(_),
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_queued(&self) -> bool {
        match &self.state {
            TaskState::Waiting { status: Waiting::Queued, .. } => true,
            _ => false,
        }
    }
}

// Get general info about task in it's context
pub enum Startability {
    Ready,
    Waiting(Duration),
    Dependencies(Vec<usize>),
    DependenciesFailure(usize),
    Stashed,
    Locked,

    Unknown,
}

pub enum GeneralState {
    Healthy,
    Failed,
    Paused,
    Waiting,
}

impl Task {
    pub fn can_be_started(&self, state: &State) -> bool {
        match self.start_info(state) {
            Startability::Ready => true,
            _ => false,
        }
    }

    pub fn start_info(&self, state: &State) -> Startability {
        match &self.state {
            TaskState::Waiting {
                locked: true, ..
            } => Startability::Locked,

            TaskState::Waiting {
                status: Waiting::Stashed,
                ..
            } => Startability::Stashed,

            TaskState::Waiting {
                status: Waiting::Queued,
                enqueue_at: Some(time),
                ..
            } if time > &Local::now() => {
                Startability::Waiting(*time - Local::now())
            }

            TaskState::Waiting {
                status: Waiting::Queued,
                dependencies,
                ..
            } => {
                let dep_not_done: Vec<_> = dependencies
                    .iter()
                    .flat_map(|id| state.get_task(*id))
                    .filter(|task| !task.is_done())
                    .collect();

                if !dep_not_done.is_empty() {
                    let failed = dep_not_done.iter().find(|task| task.is_errored());

                    if let Some(failed_task) = failed {
                        Startability::DependenciesFailure(failed_task.id)
                    } else {
                        Startability::Dependencies(dep_not_done.iter().map(|task| task.id).collect())
                    }
                } else {
                    Startability::Ready
                }
            }
            _ => Startability::Unknown,
        }
    }

    pub fn general_state(&self, state: &State) -> GeneralState {
        match &self.state {
            TaskState::Waiting { .. } => match self.start_info(state) {
                Startability::Ready
                | Startability::Waiting(_)
                | Startability::Dependencies(_)
                | Startability::Locked | Startability::Stashed => GeneralState::Waiting,
                Startability::Unknown | Startability::DependenciesFailure(_) => {
                    GeneralState::Failed
                }
            },
            TaskState::Running { status, .. } => match status {
                Running::Paused => GeneralState::Paused,
                Running::Running => GeneralState::Healthy,
            },
            TaskState::Finished { status, .. } => match status {
                Finished::Errored |
                Finished::UnableToSpawn(_) |
                Finished::Failed(_) | Finished::Killed => GeneralState::Failed,
                Finished::Done => GeneralState::Healthy,
            },
        }
    }
}

// Update tasks
impl Task {
    pub fn kill(&mut self) {
        self.terminate(Finished::Killed, String::new(), String::new());
    }

    pub fn stash(&mut self) {
        match self.state {
            TaskState::Waiting { ref mut status, .. } => {
                *status = Waiting::Stashed;
            }
            _ => (),
        }
    }

    pub fn set_enqueue_at(&mut self, enqueue_at: Option<DateTime<Local>>) {
        match self.state {
            TaskState::Waiting {
                enqueue_at: ref mut enqueue,
                ..
            } => {
                *enqueue = enqueue_at;
            }
            _ => (),
        }
    }

    pub fn queue(&mut self) {
        match self.state {
            TaskState::Waiting { ref mut status, .. } => {
                *status = Waiting::Queued;
            }
            _ => (),
        }
    }

    pub fn lock(&mut self) {
        match self.state {
            TaskState::Waiting { ref mut locked, .. } => {
                *locked = true;
            }
            _ => (),
        }
    }

    pub fn unlock(&mut self) {
        match self.state {
            TaskState::Waiting { ref mut locked, .. } => {
                *locked = false;
            }
            _ => (),
        }
    }

    pub fn clean_output(&mut self) {
        match self.state {
            TaskState::Finished {
                ref mut stdout,
                ref mut stderr,
                ..
            } => {
                *stdout = String::new();
                *stderr = String::new();
            }
            _ => (),
        }
    }

    pub fn terminate(&mut self, status: Finished, stdout: String, stderr: String) {
        self.state = TaskState::Finished {
            start: match self.state {
                TaskState::Running { start, .. } => start,
                _ => Local::now(),
            },
            end: Local::now(),
            status,
            stderr,
            stdout,
        }
    }

    pub fn start(&mut self) {
        self.state = TaskState::Running {
            status: Running::Running,
            start: Local::now(),
        }
    }

    pub fn pause(&mut self) {
        match self.state {
            TaskState::Running { ref mut status, .. } => {
                *status = Running::Paused;
            }
            _ => (),
        }
    }

    pub fn unpause(&mut self) {
        match self.state {
            TaskState::Running { ref mut status, .. } => {
                *status = Running::Running;
            }
            _ => (),
        }
    }
}
