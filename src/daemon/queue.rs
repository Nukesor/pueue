use daemon::task::Task;


/// This function is for
pub struct QueueHandler {
    queue: Vec<Task>,
    next_key: u32,
}


impl QueueHandler {
    pub fn new() -> Self {
        QueueHandler{
            queue: Vec::new(),
            next_key: 0,
        }
    }
}
