use ::failure::Error;
use ::futures::prelude::*;
use ::futures::Future;
use ::std::collections::HashMap;
use ::std::process::{Command, ExitStatus, Stdio};
use ::tokio_process::{Child, CommandExt};

use crate::daemon::queue::*;
use crate::daemon::task::{Task, TaskStatus};
use crate::file::log::{create_log_file_handles, open_log_file_handles};

pub struct TaskHandler {
    children: HashMap<usize, Box<Child>>,
    is_running: bool,
}

impl TaskHandler {
    pub fn new() -> Self {
        TaskHandler {
            children: HashMap::new(),
            is_running: true,
        }
    }
}

impl TaskHandler {
    pub fn check(&mut self, queue: &mut Queue) {
        self.process_finished(queue);
        self.check_new(queue);
    }

    /// Check whether there are any finished processes
    fn process_finished(&mut self, queue: &mut Queue) {
        let (finished, errored) = self.get_finished();
        // Iterate over everything.
        for index in finished.iter() {
            let child = self.children.remove(index).expect("Child went missing");
            handle_finished_child(queue, *index, child);
        }

        for index in errored.iter() {
            let child = self.children.remove(index).expect("Child went missing");
            change_status(queue, *index, TaskStatus::Failed);
        }
    }

    fn get_finished(&mut self) -> (Vec<usize>, Vec<usize>) {
        let mut finished = Vec::new();
        let mut errored = Vec::new();
        for (index, child) in self.children.iter_mut() {
            match child.poll() {
                // Handle a child error.
                Err(error) => {
                    println!("Task {} failed with error {:?}", index, error);
                    errored.push(index.clone());
                }
                // Child process did not error yet
                Ok(success) => {
                    match success {
                        // Child process is done
                        Async::Ready(_) => {
                            finished.push(index.clone());
                        }
                        // Child process is not done, keep waiting
                        Async::NotReady => continue,
                    }
                }
            }
        }
        (finished, errored)
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
            .spawn_async()?;
        self.children.insert(index, Box::new(child));

        Ok(())
    }
}
