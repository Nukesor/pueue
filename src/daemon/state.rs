use ::std::collections::BTreeMap;
use ::std::process::Child;
use ::std::sync::{Arc, Mutex};

use crate::daemon::task::{Task, TaskStatus};

pub type SharedState = Arc<Mutex<State>>;

pub struct State {
    max_id: i32,

    tasks: BTreeMap<i32, Task>,
}

impl State {
    pub fn new() -> State {
        return State {
            max_id: 0,
            tasks: BTreeMap::new(),
        };
    }

    pub fn add_task(&mut self, mut task: Task) {
        task.id = self.max_id;
        self.tasks.insert(self.max_id, task);
        self.max_id += 1;
    }

    pub fn remove_task(&mut self, id: i32) -> Option<Task> {
        self.tasks.remove(&id)
    }

    pub fn get_next_task(&mut self) -> Option<(i32)> {
        for (id, task) in self.tasks.iter() {
            match task.status {
                TaskStatus::Queued => {
                    return Some(*id);
                }
                _ => continue,
            }
        }
        None
    }

    pub fn change_status(&mut self, id: i32, status: TaskStatus) {
        if let Some(ref mut task) = self.tasks.get_mut(&id) {
            task.status = status;
        };
    }

    pub fn get_task_status(&mut self, id: i32) -> Option<TaskStatus> {
        if let Some(ref task) = self.tasks.get(&id) {
            return Some(task.status.clone());
        };
        None
    }

    pub fn handle_finished_child(&mut self, _index: i32, _child: Child) {}
}
