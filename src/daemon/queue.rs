use daemon::task::{Task, TaskStatus};

pub struct QueueHandler {
    queue: Vec<Option<Box<Task>>>,
}

impl QueueHandler {
    pub fn new() -> Self {
        QueueHandler { queue: Vec::new() }
    }

    pub fn add_task(&mut self, command: &String, path: &String) {
        let task = Task {
            command: command.clone(),
            path: path.clone(),
            status: TaskStatus::Queued,
            returncode: None,
            stdout: None,
            stderr: None,
            start: None,
            end: None,
        };

        self.queue.push(Some(Box::new(task)));
    }

    pub fn get_next_task(&self) -> Option<(usize, Option<&Task>)> {
        for (i, task) in self.queue.iter().enumerate() {
            match task {
                None => continue,
                Some(task) => match task.status {
                    TaskStatus::Queued => {
                        return Some((i as usize, Some(task)));
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
}
