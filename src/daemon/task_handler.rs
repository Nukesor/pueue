use daemon::queue::QueueHandler;
use daemon::task::TaskStatus;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TaskHandler {
    queue_handler: Rc<RefCell<QueueHandler>>,
}

impl TaskHandler {
    pub fn new(queue_handler: Rc<RefCell<QueueHandler>>) -> Self {
        TaskHandler {
            queue_handler: queue_handler,
        }
    }
}

impl TaskHandler {
    pub fn check_new(&mut self) {
        let index = {
            let queue_handler = self.queue_handler.borrow();
            let next = queue_handler.get_next_task();

            let (index, task) = if let Some((index, task)) = next {
                (index, task)
            } else {
                return;
            };

            index
        };

        let mut queue_handler_mut = self.queue_handler.borrow_mut();
        queue_handler_mut.change_status(index, TaskStatus::Running);
    }
}
