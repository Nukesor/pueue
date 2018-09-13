use daemon::queue::QueueHandler;
use daemon::task::TaskStatus;
use std::io::Error;
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::{Child, Command};
use std::rc::Rc;

pub struct TaskHandler {
    queue_handler: Rc<RefCell<QueueHandler>>,
    children: HashMap<usize, Box<Child>>,
}

impl TaskHandler {
    pub fn new(queue_handler: Rc<RefCell<QueueHandler>>) -> Self {
        TaskHandler {
            queue_handler: queue_handler,
            children: HashMap::new(),
        }
    }
}

impl TaskHandler {
    pub fn check(&mut self) {
        self.check_finished();
        self.check_new();
    }

    /// Check whether there are any finished processes
    fn check_finished(&mut self) {
        // Iterate over everything.
        for (index, child) in &mut self.children {
            match child.try_wait() {
                // Handle a child error.
                Err(error) => {
                    println!("Task {} failed with error {:?}", index, error);

                    self.queue_handler.borrow_mut().change_status(*index, TaskStatus::Failed);
                }
                // Child process did not error yet
                Ok(success) => {
                    match success {
                        // Child process is not done, keep waiting
                        None => continue,

                        // Child process is done
                        Some(exit_status) => {
                            self.queue_handler.borrow_mut().handle_finished_child(*index, child, exit_status);
                        }
                    }
                }
            }
        }
    }

    /// See if the task manager has a free slot and can start a new process.
    fn check_new(&mut self) {
        let (index, command, path) = {
            let queue_handler = self.queue_handler.borrow();
            let next = queue_handler.get_next_task();

            if let Some((index, command, path)) = next {
                (index, command, path)
            } else {
                return;
            }
        };

        self.start_process(index, command, path);

        self.queue_handler
            .borrow_mut()
            .change_status(index, TaskStatus::Running);
    }

    fn start_process(&mut self, index: usize, command: String, path: String) -> Result<(), Error>  {
        let child = Command::new(command).current_dir(path).spawn()?;

        self.children.insert(index, Box::new(child));

        Ok(())
    }
}
