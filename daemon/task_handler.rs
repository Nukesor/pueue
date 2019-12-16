use ::std::collections::BTreeMap;
use ::std::io::Write;
use ::std::process::Stdio;
use ::std::process::{Child, Command};
use ::std::sync::mpsc::Receiver;
use ::std::time::Duration;

use ::anyhow::Result;
use ::chrono::prelude::*;
use ::log::{debug, error, info, warn};
use ::nix::sys::signal;
use ::nix::sys::signal::Signal;
use ::nix::unistd::Pid;

use ::pueue::log::*;
use ::pueue::message::*;
use ::pueue::settings::Settings;
use ::pueue::state::SharedState;
use ::pueue::task::{Task, TaskStatus};

pub struct TaskHandler {
    state: SharedState,
    settings: Settings,
    receiver: Receiver<Message>,
    pub children: BTreeMap<usize, Child>,
    running: bool,
    reset: bool,
}

impl TaskHandler {
    pub fn new(settings: Settings, state: SharedState, receiver: Receiver<Message>) -> Self {
        let running = {
            let state = state.lock().unwrap();
            state.running
        };
        TaskHandler {
            state: state,
            settings: settings,
            receiver: receiver,
            children: BTreeMap::new(),
            running: running,
            reset: false,
        }
    }
}

/// The task handler needs to kill all child processes as soon, as the program exits
/// This is needed to prevent detached processes
impl Drop for TaskHandler {
    fn drop(&mut self) {
        let ids: Vec<usize> = self.children.keys().cloned().collect();
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
            if self.running && !self.reset {
                let _res = self.check_new();
            }
        }
    }

    /// See if the task manager has a free slot and a queued task.
    /// If that's the case, start a new process.
    fn check_new(&mut self) -> Result<()> {
        // Check while there are still slots left
        // Break the loop if no next task is found
        while self.children.len() < self.settings.daemon.default_parallel_tasks {
            match self.get_next()? {
                Some((id, task)) => self.start_process(id, &task),
                None => break,
            }
        }

        Ok(())
    }

    /// Return the next task that's queued for execution.
    /// None if no new task could be found.
    fn get_next(&mut self) -> Result<Option<(usize, Task)>> {
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
    fn start_process(&mut self, task_id: usize, task: &Task) {
        // Already get the mutex here to ensure that the task won't be manipulated
        // or removed while we are starting it over here.
        let mut state = self.state.lock().unwrap();

        // Try to get the log files to which the output of the process
        // Will be written. Error if this doesn't work!
        let (stdout_log, stderr_log) = match create_log_file_handles(task_id, &self.settings) {
            Ok((out, err)) => (out, err),
            Err(err) => {
                error!("Failed to create child log files: {:?}", err);
                return;
            }
        };

        // Spawn the actual subprocess
        let spawn_result = Command::new("sh")
            .arg("-c")
            .arg(&task.command)
            .current_dir(&task.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log))
            .spawn();

        // The spawning
        let child = match spawn_result {
            Ok(child) => child,
            Err(err) => {
                let error = format!("Failed to spawn child {} with err: {:?}", task_id, err);
                error!("{}", error);
                clean_log_handles(task_id, &self.settings);
                state.change_status(task_id, TaskStatus::Failed);
                state.add_error_message(task_id, error);
                return;
            }
        };
        self.children.insert(task_id, child);
        info!("Started task: {}", task.command);

        state.change_status(task_id, TaskStatus::Running);
    }

    /// Check whether there are any finished processes
    /// In case there are, handle them and update the shared state
    fn process_finished(&mut self) {
        let (finished, errored) = self.get_finished();
        for task_id in finished.iter() {
            let mut child = self.children.remove(task_id).expect("Child went missing");
            // Return 254, if the process has been killed by a signal
            // This is kind of dirty, but we work with it for now
            let exit_code = match child.wait().unwrap().code() {
                Some(code) => code,
                None => 254,
            };

            // Get the stdout and stderr of this task from the output files
            let (stdout, stderr) = match read_log_files(*task_id, &self.settings) {
                Ok((stdout, stderr)) => (Some(stdout), Some(stderr)),
                Err(err) => {
                    error!(
                        "Failed reading log files for task {} with error {:?}",
                        task_id, err
                    );
                    (None, None)
                }
            };
            // Now remove the output files. Don't do anything if this fails.
            // This is something the user must take care of.
            clean_log_handles(*task_id, &self.settings);

            let mut state = self.state.lock().unwrap();
            let mut task = state.tasks.get_mut(&task_id).unwrap();
            task.status = TaskStatus::Done;
            task.stdout = stdout;
            task.stderr = stderr;

            task.exit_code = Some(exit_code);
            task.end = Some(Local::now());

            state.save()
        }

        for task_id in errored.iter() {
            let _child = self.children.remove(task_id).expect("Child went missing");
            let mut state = self.state.lock().unwrap();
            state.change_status(*task_id, TaskStatus::Failed);
        }

        // The daemon got a reset request and all children just finished
        if self.reset && self.children.is_empty() {
            let mut state = self.state.lock().unwrap();
            state.reset();
            self.reset = false;
        }
    }

    /// Gather all finished tasks and sort them by finished and errored.
    /// Returns two lists of task ids, namely finished_task_ids and errored _task_ids
    fn get_finished(&mut self) -> (Vec<usize>, Vec<usize>) {
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
            Message::Send(message) => self.send(message),
            Message::Parallel(amount) => self.allow_parallel_tasks(amount),
            Message::Reset => self.reset(),
            _ => info!("Received unhandled message {:?}", message),
        }
    }

    /// Send a signal to a unix process
    fn send_signal(&mut self, id: usize, signal: Signal) -> Result<bool, nix::Error> {
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
        // Only start specific tasks
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
        let keys: Vec<usize> = self.children.keys().cloned().collect();
        for id in keys {
            self.continue_task(id);
        }
        info!("Resuming daemon (start)");
        {
            let mut state = self.state.lock().unwrap();
            state.running = true;
            self.running = true;
            state.save();
        }
    }

    /// Send a start signal to a paused task to continue execution
    fn continue_task(&mut self, id: usize) {
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
        let keys: Vec<usize> = self.children.keys().cloned().collect();
        if !message.wait {
            for id in keys {
                self.pause_task(id);
            }
        }
        info!("Pausing daemon");
        {
            let mut state = self.state.lock().unwrap();
            state.running = false;
            self.running = false;
            state.save();
        }
    }

    /// Pause a specific task.
    /// Send a signal to the process to actually pause the OS process
    fn pause_task(&mut self, id: usize) {
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
        // Only pause specific tasks
        if !message.task_ids.is_empty() {
            for id in message.task_ids {
                self.kill_task(id);
            }
            return;
        }

        // Pause the daemon and all tasks
        info!("Killing all spawned children");
        let keys: Vec<usize> = self.children.keys().cloned().collect();
        for id in keys {
            self.kill_task(id);
        }
    }

    /// Kill a specific task and handle it accordingly
    /// Triggered on `reset` and `kill`.
    fn kill_task(&mut self, task_id: usize) {
        if let Some(child) = self.children.get_mut(&task_id) {
            match child.kill() {
                Err(_) => debug!("Task {} has already finished by itself", task_id),
                _ => (),
            };
        } else {
            warn!("Tried to kill non-existing child: {}", task_id);
        }
    }

    /// Send some input to a child process
    fn send(&mut self, message: SendMessage) {
        let task_id = message.task_id;
        let input = message.input;
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

    /// Kill all children by reusing the `kill` function
    /// Set the `reset` flag, which will prevent new tasks from being spawned.
    /// If all children finished, the state will be completely reset.
    fn reset(&mut self) {
        let message = KillMessage {
            task_ids: Vec::new(),
            all: true,
        };
        self.kill(message);

        self.reset = true;
    }

    /// Adjust the amount of allowed parallel tasks
    /// This function also saves the new settings to the default config location
    fn allow_parallel_tasks(&mut self, amount: usize) {
        self.settings.daemon.default_parallel_tasks = amount;
    }
}
