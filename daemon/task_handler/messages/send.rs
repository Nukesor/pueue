use std::io::Write;

use log::{error, warn};

use crate::task_handler::TaskHandler;

impl TaskHandler {
    /// Send some input to a child process' stdin.
    pub fn send(&mut self, task_id: usize, input: String) {
        let child = match self.children.get_child_mut(task_id) {
            Some(child) => child,
            None => {
                warn!("Task {task_id} finished before input could be sent");
                return;
            }
        };
        {
            let child_stdin = child.stdin.as_mut().unwrap();
            if let Err(err) = child_stdin.write_all(&input.clone().into_bytes()) {
                error!("Failed to send input to task {task_id} with err {err:?}");
            };
        }
    }
}
