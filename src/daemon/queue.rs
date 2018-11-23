use communication::message::*;
use daemon::task::{Task, TaskStatus};
use std::process::{Child, ExitStatus};

pub type Queue = Vec<Option<Box<Task>>>;

pub fn add_task(queue: &mut Queue, add_message: AddMessage) {
    let task = Task {
        command: add_message.command.clone(),
        path: add_message.path.clone(),
        status: TaskStatus::Queued,
        returncode: None,
        stdout: None,
        stderr: None,
        start: None,
        end: None,
    };

    queue.push(Some(Box::new(task)));
}

pub fn get_next_task(queue: &mut Queue) -> Option<(usize, String, String)> {
    for (i, task) in queue.iter().enumerate() {
        match task {
            None => continue,
            Some(task) => match task.status {
                TaskStatus::Queued => {
                    return Some((i as usize, task.command.clone(), task.path.clone()));
                }
                _ => continue,
            },
        }
    }

    None
}

pub fn change_status(queue: &mut Queue, index: usize, status: TaskStatus) {
    let ref mut task = if let Some(ref mut task) = queue[index] {
        task
    } else {
        return;
    };

    task.status = status;
}

pub fn handle_finished_child(queue: &mut Queue, index: usize, child: &Child, exit_status: ExitStatus) {}
