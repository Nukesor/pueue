use ::std::collections::BTreeMap;
use ::std::process::Stdio;
use ::std::process::{Child, Command};
use ::std::sync::mpsc::Receiver;
use ::std::time::Duration;

use ::anyhow::Result;
use ::log::{debug, error, info, warn};
use ::nix::sys::signal;
use ::nix::sys::signal::Signal;
use ::nix::unistd::Pid;

use ::pueue::communication::message::*;
use ::pueue::file::log::create_log_file_handles;
use ::pueue::settings::Settings;
use ::pueue::state::SharedState;
use ::pueue::task::{Task, TaskStatus};

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

/// The task handler needs to kill all child processes as soon, as the program exits
/// This is needed to prevent detached processes
impl Drop for TaskHandler {
    fn drop(&mut self) {
        let ids: Vec<i32> = self.children.keys().cloned().collect();
        for id in ids {
            let mut child = self.children.remove(&id).expect("Failed killing children");
            info!("Killing child {}", id);
            let _ = child.kill();
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
        // Check while there are still slots left
        // Break the loop if no next task is found
        while self.children.len() < self.settings.daemon.default_worker_count {
            match self.get_next()? {
                Some((id, task)) => self.start_process(id, &task)?,
                None => break,
            }
        }

        Ok(())
    }

    /// Return the next task that's queued for execution.
    /// None if no new task could be found.
    fn get_next(&mut self) -> Result<Option<(i32, Task)>> {
        let mut state = self.state.lock().unwrap();
        match state.get_next_task() {
            Some(id) => {
                let task = state.get_task_clone(id).unwrap();
                Ok(Some((id, task)))
            }
            None => Ok(None),
        }
    }

    /// Actually spawn a new sub process
    /// The output of subprocesses is piped into a seperate file for easier access
    fn start_process(&mut self, id: i32, task: &Task) -> Result<()> {
        // Already get the mutex here to ensure that the task won't be manipulated
        // or removed while we are starting it over here.
        let mut state = self.state.lock().unwrap();

        // Spawn the actual subprocess
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

        state.change_status(id, TaskStatus::Running);

        Ok(())
    }

    /// Check whether there are any finished processes
    /// In case there are, handle them and update the shared state
    fn process_finished(&mut self) {
        let (finished, errored) = self.get_finished();
        for id in finished.iter() {
            let child = self.children.remove(id).expect("Child went missing");
            {
                let mut state = self.state.lock().unwrap();
                state.handle_finished_child(*id, child);
            }
        }

        for id in errored.iter() {
            let _child = self.children.remove(id).expect("Child went missing");
            {
                let mut state = self.state.lock().unwrap();
                state.change_status(*id, TaskStatus::Failed);
            }
        }
    }

    /// Gather all finished tasks and sort them by finished and errored.
    /// Returns two lists of task ids, namely finished_task_ids and errored _task_ids
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
                    info!("Task {} just finished", id);
                    finished.push(*id);
                }
            }
        }
        (finished, errored)
    }

    /// Some client instructions require immediate action by the task handler
    /// These commands are
    fn receive_commands(&mut self) {
        let timeout = Duration::from_millis(100);
        // Don't use recv_timeout for now, until this bug get's fixed
        // https://github.com/rust-lang/rust/issues/39364
        //match self.receiver.recv_timeout(timeout) {
        std::thread::sleep(timeout);
        match self.receiver.try_recv() {
            Ok(message) => self.handle_message(message),
            Err(_) => {}
        };
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::Pause(message) => self.pause(message),
            Message::Start(message) => self.start(message),
            Message::Kill(message) => self.kill(message),
            _ => info!("Received unhandled message {:?}", message),
        }
    }

    /// Send a signal to a unix process
    fn send_signal(&mut self, id: i32, signal: Signal) -> Result<bool, nix::Error> {
        if let Some(child) = self.children.get(&id) {
            debug!("Sending signal {} to {}", signal, id);
            let pid = Pid::from_raw(child.id() as i32);
            signal::kill(pid, signal)?;
            return Ok(true);
        };

        error!(
            "Tried to send signal {} to non existing child {}",
            signal, id
        );
        Ok(false)
    }

    /// Handle the start message:
    /// 1. Either start the daemon and all tasks.
    /// 2. Or force the start of specific tasks.
    fn start(&mut self, message: StartMessage) {
        // Only pause specific tasks
        if let Some(task_ids) = message.task_ids {
            for id in task_ids {
                // Continue all children that are simply paused
                if self.children.contains_key(&id) {
                    self.continue_task(id);
                } else {
                    // Start processes for all tasks that haven't been started yet
                    let task = {
                        let mut state = self.state.lock().unwrap();
                        state.get_task_clone(id)
                    };

                    if let Some(task) = task {
                        self.start_process(id, &task);
                    }
                }
            }
            return;
        }

        // Start the daemon and all paused tasks
        info!("Resuming daemon (start)");
        self.is_running = true;
        let keys: Vec<i32> = self.children.keys().cloned().collect();
        for id in keys {
            self.continue_task(id);
        }
    }

    /// Send a start signal to a paused task to continue execution
    fn continue_task(&mut self, id: i32) {
        if !self.children.contains_key(&id) {
            return;
        }
        match self.send_signal(id, Signal::SIGCONT) {
            Err(err) => warn!("Failed starting task {}: {:?}", id, err),
            Ok(success) => {
                if success {
                    let mut state = self.state.lock().unwrap();
                    state.change_status(id, TaskStatus::Running);
                }
            }
        }
    }

    /// Handle the pause message:
    /// 1. Either pause the daemon and all tasks.
    /// 2. Or only pause specific tasks.
    fn pause(&mut self, message: PauseMessage) {
        // Only pause specific tasks
        if let Some(task_ids) = message.task_ids {
            for id in task_ids {
                self.pause_task(id);
            }
            return;
        }

        // Pause the daemon and all tasks
        info!("Pausing daemon");
        self.is_running = false;
        let keys: Vec<i32> = self.children.keys().cloned().collect();
        if !message.wait {
            for id in keys {
                self.pause_task(id);
            }
        }
    }

    /// Pause a specific task.
    /// Send a signal to the process to actually pause the OS process
    fn pause_task(&mut self, id: i32) {
        if !self.children.contains_key(&id) {
            return;
        }
        match self.send_signal(id, Signal::SIGSTOP) {
            Err(err) => info!("Failed pausing task {}: {:?}", id, err),
            Ok(success) => {
                if success {
                    let mut state = self.state.lock().unwrap();
                    state.change_status(id, TaskStatus::Paused);
                }
            }
        }
    }

    /// Handle the pause message:
    /// 1. Either kill all tasks.
    /// 2. Or only kill specific tasks.
    fn kill(&mut self, message: KillMessage) {
        println!("lol");
        // Only pause specific tasks
        if !message.task_ids.is_empty() {
            for id in message.task_ids {
                self.kill_task(id);
            }
            return;
        }

        // Pause the daemon and all tasks
        info!("Killing all spawned children");
        let keys: Vec<i32> = self.children.keys().cloned().collect();
        for id in keys {
            self.kill_task(id);
        }
    }

    /// Kill a specific task and handle it accordingly
    /// Triggered on `reset` and `kill`.
    fn kill_task(&mut self, task_id: i32) {
        if let Some(child) = self.children.get_mut(&task_id) {
            match child.kill() {
                Err(_) => debug!("Task {} has already finished by itself", task_id),
                _ => (),
            };
        } else {
            warn!("Tried to kill non-existing child: {}", task_id);
        }
    }
}
