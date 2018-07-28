use std::rc::Rc;
use std::cell::RefCell;
use daemon::queue::QueueHandler;

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
        let mut queue_handler = self.queue_handler.borrow_mut();
        let task = queue_handler.get_next_task();


    }
}
