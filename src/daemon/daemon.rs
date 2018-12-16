use ::failure::Error;
use ::futures::prelude::*;
use ::futures::Future;
use ::std::collections::HashMap;

use crate::communication::message::*;
use crate::daemon::queue::*;
use crate::daemon::task_handler::TaskHandler;
use crate::daemon::socket_handler::SocketHandler;
use crate::settings::Settings;

/// The daemon is center of all logic in pueue.
/// This is the single source of truth for all clients and workers.
pub struct Daemon {
    queue: Queue,
    task_handler: TaskHandler,
    socket_handler: SocketHandler,
}

impl Daemon {
    /// Create a new daemon.
    /// This function also handle the creation of other components,
    /// such as the queue, sockets and the process handler.
    pub fn new(settings: &Settings) -> Self {
        let task_handler = TaskHandler::new();
        let socket_handler = SocketHandler::new(settings);

        Daemon {
            queue: Vec::new(),
            task_handler: task_handler,
            socket_handler: socket_handler,
        }
    }
}

impl Daemon {
    pub fn handle_instructions(&mut self, instructions: HashMap<MessageType, String>) {
        for (instruction_type, instruction) in instructions {
            let message = extract_message(instruction_type.clone(), instruction);

            match instruction_type {
                MessageType::Add => {
                    let add_message = if let Some(add_message) = message.add {
                        add_message
                    } else {
                        panic!("Error in add message unwrap.");
                    };
                    add_task(&mut self.queue, add_message);
                }
                MessageType::Invalid => panic!("Invalid message type"),
            };
        }
    }
}

impl Future for Daemon {
    type Item = ();
    type Error = Error;

    /// The poll function of the daemon.
    /// This is continuously called by the Tokio core.
    fn poll(&mut self) -> Result<Async<()>, Self::Error> {
        // Accept all new connections
        self.socket_handler.accept_incoming();

        // Poll all connection futures and handle the received instruction.
        let instructions = self.socket_handler.handle_incoming();

        self.handle_instructions(instructions);

        self.socket_handler.handle_responses();

        self.task_handler.check(&mut self.queue);

        // `NotReady` is returned here because the future never actually
        // completes. The server runs until it is dropped.
        Ok(Async::NotReady)
    }
}
