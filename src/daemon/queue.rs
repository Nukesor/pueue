use ::std::mem;
use ::std::process::ExitStatus;
use ::tokio_process::Child;

use crate::communication::message::*;
use crate::daemon::task::{Task, TaskStatus};

pub type Queue = Vec<Option<Box<Task>>>;

pub fn add_task(queue: &mut Queue, message: AddMessage) {
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

pub fn handle_finished_child(
    _queue: &mut Queue,
    _index: usize,
    _child: Box<Child>,
) {
}
