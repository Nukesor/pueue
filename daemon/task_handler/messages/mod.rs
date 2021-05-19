use std::thread::sleep;
use std::time::Duration;

use log::info;

use pueue_lib::network::message::*;

use crate::task_handler::TaskHandler;

mod kill;
mod pause;
mod send;
mod start;

impl TaskHandler {
    /// Some client instructions require immediate action by the task handler
    /// This function is also responsible for waiting
    pub fn receive_messages(&mut self) {
        // Sleep for a few milliseconds. We don't want to hurt the CPU.
        let timeout = Duration::from_millis(200);
        // Don't use recv_timeout for now, until this bug get's fixed.
        // https://github.com/rust-lang/rust/issues/39364
        //match self.receiver.recv_timeout(timeout) {
        sleep(timeout);

        if let Ok(message) = self.receiver.try_recv() {
            self.handle_message(message);
        };
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::Pause(message) => self.pause(
                message.task_ids,
                message.group,
                message.all,
                message.children,
                message.wait,
            ),
            Message::Start(message) => self.start(
                message.task_ids,
                message.group,
                message.all,
                message.children,
            ),
            Message::Kill(message) => self.kill(
                message.task_ids,
                message.group,
                message.all,
                message.children,
                message.signal,
            ),
            Message::Send(message) => self.send(message.task_id, message.input),
            Message::Reset(message) => self.reset(message.children),
            Message::DaemonShutdown => {
                info!("Killing all children due to graceful shutdown.");
                self.graceful_shutdown = true;
                self.reset(false);
            }
            _ => info!("Received unhandled message {:?}", message),
        }
    }
}
