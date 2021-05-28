use std::io::Write;

use log::warn;

use crate::task_handler::TaskHandler;

impl TaskHandler {
    /// Send some input to a child process' stdin.
    pub fn send(&mut self, task_id: usize, input: String) {
        let child = match self.children.get_mut(&task_id) {
            Some(child) => child,
            None => {
                warn!(
                    "Task {} finished before input could be sent: {}",
                    task_id, input
                );
                return;
            }
        };
        {
            let child_stdin = child.stdin.as_mut().unwrap();
            if let Err(err) = child_stdin.write_all(&input.clone().into_bytes()) {
                warn!(
                    "Failed to send input to task {} with err {:?}: {}",
                    task_id, err, input
                );
            };
        }
    }
}
