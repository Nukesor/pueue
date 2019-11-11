use ::std::sync::{Arc, Mutex};
use ::std::collections::BTreeMap;
use ::anyhow::{Error, Result};
use ::std::process::Child;

use crate::communication::message::*;
use crate::daemon::task::{Task, TaskStatus};
use crate::daemon::task_handler::*;

pub type SharedState = Arc<Mutex<State>>;

pub struct State {
    max_id: i32,

    pub queued: BTreeMap<i32, Task>,
    pub running: BTreeMap<i32, Task>,
    pub finished: BTreeMap<i32, Task>,
}

impl State {
    pub fn add_task(&mut self, message: AddMessage) -> Result<Message> {
        let mut command = message.command.clone();
        let arguments = command.split_off(1);
        let task = Task {
            id: self.max_id,
            command: command.pop().expect("Expected command"),
            arguments: arguments,
            path: message.path.clone(),
            status: TaskStatus::Queued,
            returncode: None,
            stdout: None,
            stderr: None,
            start: None,
            end: None,
        };

        self.queued.insert(self.max_id, task);
        self.max_id += 1;

        create_success_message(String::from("New task added."))
    }

    pub fn remove_task(
        &mut self,
        task_handler: &mut TaskHandler,
        message: RemoveMessage,
    ) -> Result<Message> {
        create_success_message(String::from("Task removed"))
    }

    pub fn get_next_task(&mut self) -> Option<(i32)> {
        for (id, task) in self.queued.iter() {
            match task.status {
                TaskStatus::Queued => {
                    return Some(*id);
                }
                _ => continue,
            }
        }
        None
    }

    pub fn change_status(&mut self, index: i32, status: TaskStatus) {
        let ref mut task = if let Some(ref mut task) = self.queued.get_mut(&index) {
            task.status = status;
        };
    }

    pub fn get_task_status(&mut self, index: i32) -> Option<TaskStatus> {
        if let Some(ref task) = self.queued.get(&index) {
            return Some(task.status.clone());
        };
        None
    }

    pub fn handle_finished_child(&mut self, _index: i32, _child: Child) {}
}
