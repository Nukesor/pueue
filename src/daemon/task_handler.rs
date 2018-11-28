use ::failure::Error;
use ::std::collections::HashMap;
use ::std::process::{Child, Command, Stdio};
use ::tokio_process::CommandExt;

use crate::daemon::queue::*;
use crate::daemon::task::{Task, TaskStatus};
use crate::file::log::{create_log_file_handles, open_log_file_handles};

pub struct TaskHandler {
    children: HashMap<usize, Box<Child>>,
}

impl TaskHandler {
    pub fn new() -> Self {
        TaskHandler {
            children: HashMap::new(),
        }
    }
}

impl TaskHandler {
    pub fn check(&mut self, queue: &mut Queue) {
        self.check_finished(queue);
        self.check_new(queue);
    }

    /// Check whether there are any finished processes
    fn check_finished(&mut self, queue: &mut Queue) {
        // Iterate over everything.
        for (index, child) in &mut self.children {
            match child.try_wait() {
                // Handle a child error.
                Err(error) => {
                    println!("Task {} failed with error {:?}", index, error);

                    change_status(queue, *index, TaskStatus::Failed);
                }
                // Child process did not error yet
                Ok(success) => {
                    match success {
                        // Child process is not done, keep waiting
                        None => continue,

                        // Child process is done
                        Some(exit_status) => {
                            handle_finished_child(queue, *index, child, exit_status);
                        }
                    }
                }
            }
        }
    }

    /// See if the task manager has a free slot and can start a new process.
    fn check_new(&mut self, queue: &mut Queue) -> Result<(), Error> {
        let next_task = get_next_task(queue);
        let (index, task) = if let Some((index, task)) = next_task {
            (index, task)
        } else {
            return Ok(());
        };

        self.start_process(index, &task)?;

        change_status(queue, index, TaskStatus::Running);

        Ok(())
    }

    fn start_process(&mut self, index: usize, task: &Task) -> Result<(), Error> {
        let (stdout_log, stderr_log) = create_log_file_handles(index)?;
        let child = Command::new(task.command.clone())
            .current_dir(task.path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log))
            .spawn()?;
        self.children.insert(index, Box::new(child));

        Ok(())
    }
}
