use ::std::collections::BTreeMap;
use ::std::process::Stdio;
use ::std::process::{Child, Command};
use ::std::sync::mpsc::Receiver;
use ::std::time::Duration;

use ::anyhow::{anyhow, Result};
use ::log::info;

use crate::communication::message::Message;
use crate::daemon::state::SharedState;
use crate::daemon::task::{Task, TaskStatus};
use crate::file::log::create_log_file_handles;
use crate::settings::Settings;

pub struct TaskHandler {
    state: SharedState,
    settings: Settings,
    receiver: Receiver<Message>,
    pub children: BTreeMap<i32, Child>,
    is_running: bool,
}

impl TaskHandler {
    pub fn new(settings: Settings, state: SharedState, receiver: Receiver<Message>) -> Self {
        TaskHandler {
            state: state,
            settings: settings,
            receiver: receiver,
            children: BTreeMap::new(),
            is_running: true,
        }
    }
}

impl TaskHandler {
    /// Main loop of the task handler
    /// In here a few things happen:
    /// 1. Propagated commands from socket communication is received and handled
    /// 2. Check whether any tasks just finished
    /// 3. Check whether we can spawn new tasks
    pub fn run(&mut self) {
        loop {
            self.receive_commands();
            self.process_finished();
            if self.is_running {
                let _res = self.check_new();
            }
        }
    }

    /// See if the task manager has a free slot and a queued task.
    /// If that's the case, start a new process.
    fn check_new(&mut self) -> Result<()> {
        // All slots are already occupied
        if self.children.len() >= self.settings.daemon.default_worker_count {
            return Ok(());
        }

        if let Some((id, task)) = self.get_next()? {
            self.start_process(id, &task)?;
        }

        Ok(())
    }

    /// Return the next task that's queued for execution.
    /// None if no new task could be found.
    fn get_next(&mut self) -> Result<Option<(i32, Task)>> {
        let mut state = self.state.lock().unwrap();
        if let Some(id) = state.get_next_task() {
            let task = state.remove_task(id).ok_or(anyhow!("Expected queued item"))?;
            Ok(Some((id, task)))
        } else {
            Ok(None)
        }
    }

    /// Actually spawn a new sub process
    /// The output of subprocesses is piped into a seperate file for easier access
    fn start_process(&mut self, id: i32, task: &Task) -> Result<()> {
        let (stdout_log, stderr_log) = create_log_file_handles(id)?;
        let child = Command::new(&task.command)
            .args(&task.arguments)
            .current_dir(&task.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log))
            .spawn()?;
        self.children.insert(id, child);
        info!("Started task: {} {:?}", task.command, task.arguments);

        let mut state = self.state.lock().unwrap();
        state.change_status(id, TaskStatus::Running);

        Ok(())
    }

    fn receive_commands(&mut self) {
        let timeout = Duration::from_millis(100);
        // Don't use recv_timeout for now, until this bug get's fixed
        // https://github.com/rust-lang/rust/issues/39364
        //match self.receiver.recv_timeout(timeout) {
        std::thread::sleep(timeout);
        match self.receiver.try_recv() {
            Ok(message) => info!("{:?}", message),
            Err(_) => {},
        };
    }

    /// Check whether there are any finished processes
    fn process_finished(&mut self) {
        let (finished, errored) = self.get_finished();
        let mut state = self.state.lock().unwrap();
        // Iterate over everything.
        for id in finished.iter() {
            let child = self.children.remove(id).expect("Child went missing");
            state.handle_finished_child(*id, child);
        }

        for id in errored.iter() {
            let _child = self.children.remove(id).expect("Child went missing");
            state.change_status(*id, TaskStatus::Failed);
        }
    }

    fn get_finished(&mut self) -> (Vec<i32>, Vec<i32>) {
        let mut finished = Vec::new();
        let mut errored = Vec::new();
        for (id, child) in self.children.iter_mut() {
            match child.try_wait() {
                // Handle a child error.
                Err(error) => {
                    info!("Task {} failed with error {:?}", id, error);
                    errored.push(*id);
                }
                // Child process did not exit yet
                Ok(None) => continue,
                Ok(_exit_status) => {
                    finished.push(*id);
                }
            }
        }
        (finished, errored)
    }
}
