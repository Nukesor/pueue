use daemon::task::{Task, TaskStatus};

pub struct QueueHandler {
    queue: Vec<Option<Task>>,
}

impl QueueHandler {
    pub fn new() -> Self {
        QueueHandler {
            queue: Vec::new(),
        }
    }

    pub fn add_task(&mut self, message: String) {
        let task = Task {
            command: message,
            status: TaskStatus::Queued,
            returncode: None,
            stdout: None,
            stderr: None,
            start: None,
            end: None,
        };

        self.queue.push(Some(task));
    }

    pub fn get_next_task(&mut self) -> Option<&Task> {
        for i in 0..self.queue.len() {
            match self.queue[i] {
                None => continue,
                Some(ref task) => {
                    match task.status {
                        TaskStatus::Queued => return Some(task),
                        _ => continue,
                    }
                },
            }
        }

        None
    }
}
