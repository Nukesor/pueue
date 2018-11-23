use communication::message::*;
use daemon::task::{Task, TaskStatus};
use std::process::{Child, ExitStatus};

#[derive(Serialize, Deserialize)]
pub struct QueueHandler {
    queue: Vec<Option<Box<Task>>>,
}

impl QueueHandler {
    pub fn new() -> Self {
        QueueHandler { queue: Vec::new() }
    }

    pub fn add_task(&mut self, add_message: AddMessage) {
        let task = Task {
            command: add_message.command.clone(),
            path: add_message.path.clone(),
            status: TaskStatus::Queued,
            returncode: None,
            stdout: None,
            stderr: None,
            start: None,
            end: None,
        };

        self.queue.push(Some(Box::new(task)));
    }

    pub fn get_next_task(&self) -> Option<(usize, String, String)> {
        for (i, task) in self.queue.iter().enumerate() {
            match task {
                None => continue,
                Some(task) => match task.status {
                    TaskStatus::Queued => {
                        return Some((i as usize, task.command.clone(), task.path.clone()));
                    }
                    _ => continue,
                },
            }
        }

        None
    }

    pub fn change_status(&mut self, index: usize, status: TaskStatus) {
        let ref mut task = if let Some(ref mut task) = self.queue[index] {
            task
        } else {
            return;
        };

        task.status = status;
    }

    pub fn handle_finished_child(&mut self, index: usize, child: &Child, exit_status: ExitStatus) {}
}
