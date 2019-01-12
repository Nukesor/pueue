use ::std::mem;
use ::tokio_process::Child;

use crate::communication::message::*;
use crate::daemon::error::DaemonError;
use crate::daemon::task::{Task, TaskStatus};
use crate::daemon::task_handler::*;

pub type Queue = Vec<Option<Box<Task>>>;

pub fn add_task(queue: &mut Queue, message: AddMessage) -> Result<Message, DaemonError> {
    let task = Task {
        command: message.command.clone(),
        path: message.path.clone(),
        status: TaskStatus::Queued,
        returncode: None,
        stdout: None,
        stderr: None,
        start: None,
        end: None,
    };

    queue.push(Some(Box::new(task)));

    create_success_message(String::from("New task added."))
}

pub fn remove_task(
    queue: &mut Queue,
    task_handler: &mut TaskHandler,
    message: RemoveMessage,
) -> Result<Message, DaemonError> {
    create_success_message(String::from("Task removed"))
}

pub fn get_next_task(queue: &mut Queue) -> Option<(usize, Task)> {
    for (i, task) in queue.iter().enumerate() {
        match task {
            None => continue,
            Some(task) => match task.status {
                TaskStatus::Queued => {
                    return Some((i, *task.clone()));
                }
                _ => continue,
            },
        }
    }

    None
}

pub fn update_task(queue: &mut Queue, index: usize, task: Task) {
    mem::replace(&mut queue[index], Some(Box::new(task)));
}

pub fn change_status(queue: &mut Queue, index: usize, status: TaskStatus) {
    let ref mut task = if let Some(ref mut task) = queue[index] {
        task
    } else {
        return;
    };

    task.status = status;
}

pub fn get_task_status(queue: &Queue, index: usize) -> Option<TaskStatus> {
    if let Some(ref task) = queue[index] {
        Some(task.status.clone())
    } else {
        None
    }
}

pub fn handle_finished_child(_queue: &mut Queue, _index: usize, _child: Box<Child>) {}
