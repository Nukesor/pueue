use ::std::collections::BTreeMap;
use ::std::process::Child;
use ::std::sync::{Arc, Mutex};

use crate::daemon::task::{Task, TaskStatus};

pub type SharedState = Arc<Mutex<State>>;

pub struct State {
    max_id: i32,

    pub queued: BTreeMap<i32, Task>,
    pub running: BTreeMap<i32, Task>,
    pub finished: BTreeMap<i32, Task>,
}

impl State {
    pub fn new() -> State {
        return State {
            max_id: 0,
            queued: BTreeMap::new(),
            running: BTreeMap::new(),
            finished: BTreeMap::new(),
        };
    }

    pub fn add_task(&mut self, mut task: Task) {
        task.id = self.max_id;
        self.queued.insert(self.max_id, task);
        self.max_id += 1;
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
        if let Some(ref mut task) = self.queued.get_mut(&index) {
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
