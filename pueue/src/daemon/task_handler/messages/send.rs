use std::io::Write;

use log::{error, warn};

use crate::daemon::task_handler::TaskHandler;

impl TaskHandler {
    /// Send some input to a child process' stdin.
    pub fn send(&mut self, task_id: usize, input: String) {
        let mut state = self.state.lock().unwrap();
        let child = match state.children.get_child_mut(task_id) {
            Some(child) => child,
            None => {
                warn!("Task {task_id} finished before input could be sent");
                return;
            }
        };
        {
            let child_stdin = child.inner().stdin.as_mut().unwrap();
            if let Err(err) = child_stdin.write_all(&input.into_bytes()) {
                error!("Failed to send input to task {task_id} with err {err:?}");
            };
        }
    }
}
