use ::failure::Error;
use ::futures::prelude::*;
use ::futures::Future;
use ::std::collections::HashMap;
use ::uuid::Uuid;

use crate::communication::message::*;
use crate::daemon::queue::*;
use crate::daemon::socket_handler::SocketHandler;
use crate::daemon::task_handler::TaskHandler;
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
    pub fn handle_instructions(&mut self, mut instructions: Vec<(Uuid, String)>) -> Vec<(Uuid, Message)> {
        let mut responses: Vec<(Uuid, Message)> = Vec::new();
        while let Some((uuid, instruction)) = instructions.pop() {
            let message_in = if let Ok(message_in) = serde_json::from_str::<Message>(&instruction) {
                message_in
            } else {
                let message_out = create_failure_message(String::from("Error during message deserialization"));
                responses.push((uuid, message_out));
                continue
            };

            let result = match message_in {
                Message::Add(message_in) => {
                    add_task(&mut self.queue, message_in)
                }
                Message::Remove(message_in) => {
                    remove_task(&mut self.queue, &mut self.task_handler, message_in)
                }
                _ => {
                    Ok(Message::Failure(FailureMessage{text: String::from("Unhandled message type.")}))
                }
            };

            match result {
                Ok(message_out) => {
                    responses.push((uuid, message_out));
                },
                Err(error) => {
                    let text = format!("{:?}", error);
                    let message_out = create_failure_message(text);
                    responses.push((uuid, message_out));
                }
            }
        }

        responses
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

        // Check for finished responses
        self.socket_handler.check_responses();

        let responses = self.handle_instructions(instructions);
        println!("{:?}", responses);
        self.socket_handler.process_responses(responses);

        self.task_handler.check(&mut self.queue);

        // `NotReady` is returned here because the future never actually
        // completes. The server runs until it is dropped.
        Ok(Async::NotReady)
    }
}
