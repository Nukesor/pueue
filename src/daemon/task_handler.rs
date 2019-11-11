use ::std::collections::BTreeMap;
use ::std::process::{ExitStatus, Stdio};
use ::std::process::{Command, Child};
use ::std::time::Duration;
use ::std::sync::mpsc::Receiver;

use ::anyhow::{Error, Result, anyhow};

use crate::daemon::state::SharedState;
use crate::daemon::task::{Task, TaskStatus};
use crate::communication::message::Message;
use crate::file::log::create_log_file_handles;

pub struct TaskHandler {
    state: SharedState,
    receiver: Receiver<Message>,
    children: BTreeMap<i32, Child>,
    is_running: bool,
}

impl TaskHandler {
    pub fn new(state: SharedState, receiver: Receiver<Message>) -> Self {
        TaskHandler {
            state: state,
            receiver: receiver,
            children: BTreeMap::new(),
            is_running: true,
        }
    }
}

impl TaskHandler {
    pub fn run(&mut self) {
        loop {
            self.receive_commands();
            self.process_finished();
            self.check_new();
        }
    }

    fn receive_commands(&mut self) {
        let timeout = Duration::from_millis(250);
        match self.receiver.recv_timeout(timeout) {
            Ok(message) => println!("{:?}", message),
            Err(err) => ()
        };
    }

    /// Check whether there are any finished processes
    fn process_finished(&mut self) {
        let (finished, errored) = self.get_finished();
        let mut state = self.state.lock().unwrap();
        // Iterate over everything.
        for index in finished.iter() {
            let child = self.children.remove(index).expect("Child went missing");
            state.handle_finished_child(*index, child);
        }

        for index in errored.iter() {
            let child = self.children.remove(index).expect("Child went missing");
            state.change_status(*index, TaskStatus::Failed);
        }
    }

    fn get_finished(&mut self) -> (Vec<i32>, Vec<i32>) {
        let mut finished = Vec::new();
        let mut errored = Vec::new();
        for (index, child) in self.children.iter_mut() {
            match child.try_wait() {
                // Handle a child error.
                Err(error) => {
                    println!("Task {} failed with error {:?}", index, error);
                    errored.push(index.clone());
                }
                // Child process did not exit yet
                Ok(None) => continue,
                Ok(exit_status) => {
                    finished.push(index.clone());
                }
            }
        }
        (finished, errored)
    }

    /// See if the task manager has a free slot and can start a new process.
    fn check_new(&mut self) -> Result<()> {
        let (index, task) = if let Some((index, task)) = self.get_next()? {
            (index, task)
        } else {
            return Ok(());
        };

        self.start_process(index, &task)?;

        Ok(())
    }

    fn start_process(&mut self, index: i32, task: &Task) -> Result<()> {
        let (stdout_log, stderr_log) = create_log_file_handles(index)?;
        let child = Command::new(task.command.clone())
            .args(task.arguments.clone())
            .current_dir(task.path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log))
            .spawn()?;
        self.children.insert(index, child);

        let mut state = self.state.lock().unwrap();
        state.change_status(index, TaskStatus::Running);

        Ok(())
    }

    fn get_next(&mut self) -> Result<Option<(i32, Task)>> {
        let mut state = self.state.lock().unwrap();
        let next_task = state.get_next_task();
        match next_task {
            Some(index) => {
                let task = state.queued.remove(&index).ok_or(anyhow!("Expected queued item"))?;
                Ok(Some((index, task)))
            }
            None => Ok(None)
        }
    }
}
