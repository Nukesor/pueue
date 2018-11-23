use daemon::task::TaskStatus;
use daemon::queue::*;
use failure::Error;
use std::collections::HashMap;
use std::process::{Child, Command};

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
                            handle_finished_child(
                                queue,
                                *index,
                                child,
                                exit_status,
                            );
                        }
                    }
                }
            }
        }
    }

    /// See if the task manager has a free slot and can start a new process.
    fn check_new(&mut self, queue: &mut Queue) -> Result<(), Error> {
        let (index, command, path) = {
            let next = get_next_task(queue);

            if let Some((index, command, path)) = next {
                (index, command, path)
            } else {
                return Ok(());
            }
        };

        self.start_process(index, command, path)?;

        change_status(queue, index, TaskStatus::Running);

        Ok(())
    }

    fn start_process(&mut self, index: usize, command: String, path: String) -> Result<(), Error> {
        let child = Command::new(command).current_dir(path).spawn()?;

        self.children.insert(index, Box::new(child));

        Ok(())
    }
}
