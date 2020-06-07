use ::anyhow::{anyhow, Context, Result};
use ::async_std::net::TcpStream;
use ::log::error;
use ::std::collections::HashMap;
use ::std::env::{current_dir, vars};
use ::std::io::{self, Write};

use crate::cli::{Opt, SubCommand};
use crate::edit::*;
use crate::output::*;
use ::pueue::message::*;
use ::pueue::protocol::*;
use ::pueue::settings::Settings;
use ::pueue::state::State;

/// Representation of a client.
/// For convenience purposes this logic has been wrapped in a struct.
/// The client is responsible for connecting to the daemon, sending an instruction
/// and interpreting the response.
///
/// Most commands are a simple ping-pong. Though, some commands require a more complex
/// communication pattern (e.g. `show -f`, which contiuously streams the output of a task).
pub struct Client {
    opt: Opt,
    daemon_address: String,
    pub settings: Settings,
}

impl Client {
    pub fn new(settings: Settings, opt: Opt) -> Result<Self> {
        // // Commandline argument overwrites the configuration files values for address
        // let address = if let Some(address) = opt.address.clone() {
        //     address
        // } else {
        //     settings.client.daemon_address
        // };

        // Commandline argument overwrites the configuration files values for port
        let port = if let Some(port) = opt.port.clone() {
            port
        } else {
            settings.client.daemon_port.clone()
        };

        // Don't allow anything else than loopback until we have proper crypto
        // let address = format!("{}:{}", address, port);
        let address = format!("127.0.0.1:{}", port);

        Ok(Client {
            opt,
            daemon_address: address,
            settings,
        })
    }

    pub async fn connect(&self) -> Result<TcpStream> {
        // Connect to socket
        let mut socket = TcpStream::connect(&self.daemon_address)
            .await
            .context("Failed to connect to the daemon. Did you start it?")?;

        let secret = self.settings.client.secret.clone().into_bytes();
        send_bytes(secret, &mut socket).await?;

        Ok(socket)
    }

    pub async fn send(&self, message: Message, socket: &mut TcpStream) -> Result<()> {
        // Create the message payload and send it to the daemon.
        send_message(message, socket).await?;

        // Check if we can receive the response from the daemon
        let mut message = receive_message(socket).await?;

        while self.handle_message(message, socket).await? {
            // Check if we can receive the response from the daemon
            message = receive_message(socket).await?;
        }

        Ok(())
    }

    /// Most returned messages can be handled in a generic fashion.
    /// However, some commands need some ping-pong or require continuous receiving of messages.
    ///
    /// If this function returns `Ok(true)`, the parent function will continue to receive
    /// and handle messages from the daemon. Otherwise the client will simply exit.
    async fn handle_message(&self, message: Message, socket: &mut TcpStream) -> Result<bool> {
        match message {
            Message::Success(text) => print_success(text),
            Message::Failure(text) => print_error(text),
            Message::StatusResponse(state) => print_state(state, &self.opt.cmd),
            Message::LogResponse(task_logs) => print_logs(task_logs, &self.opt.cmd, &self.settings),
            Message::EditResponse(message) => {
                // Create a new message with the edited command
                let message = edit(message, &self.opt.cmd);
                send_message(message, socket).await?;
                return Ok(true);
            }
            Message::Stream(text) => {
                print!("{}", text);
                io::stdout().flush().unwrap();
                return Ok(true);
            }
            _ => error!("Received unhandled response message"),
        };

        Ok(false)
    }

    /// Convert the cli command into the message that's being sent to the server,
    /// so it can be understood by the daemon.
    pub async fn get_message_from_opt(&self, socket: &mut TcpStream) -> Result<Message> {
        match &self.opt.cmd {
            SubCommand::Add {
                command,
                start_immediately,
                stashed,
                group,
                delay_until,
                dependencies,
            } => {
                let cwd_pathbuf = current_dir()?;
                let cwd = cwd_pathbuf
                    .to_str()
                    .context("Cannot parse current working directory (Invalid utf8?)")?;

                let mut envs = HashMap::new();
                // Save all environment variables for later injection into the started task
                for (key, value) in vars() {
                    envs.insert(key, value);
                }

                Ok(Message::Add(AddMessage {
                    command: command.join(" "),
                    path: cwd.to_string(),
                    envs,
                    start_immediately: *start_immediately,
                    stashed: *stashed,
                    group: group.clone(),
                    enqueue_at: *delay_until,
                    dependencies: dependencies.to_vec(),
                }))
            }
            SubCommand::Remove { task_ids } => Ok(Message::Remove(task_ids.clone())),
            SubCommand::Stash { task_ids } => Ok(Message::Stash(task_ids.clone())),
            SubCommand::Switch {
                task_id_1,
                task_id_2,
            } => {
                let message = SwitchMessage {
                    task_id_1: *task_id_1,
                    task_id_2: *task_id_2,
                };
                Ok(Message::Switch(message))
            }
            SubCommand::Enqueue {
                task_ids,
                delay_until,
            } => {
                let message = EnqueueMessage {
                    task_ids: task_ids.clone(),
                    enqueue_at: *delay_until,
                };
                Ok(Message::Enqueue(message))
            }
            SubCommand::Start {
                task_ids,
                group,
                all,
                children,
            } => {
                let message = StartMessage {
                    task_ids: task_ids.clone(),
                    group: group.clone(),
                    all: *all,
                    children: *children,
                };
                Ok(Message::Start(message))
            }
            SubCommand::Restart {
                task_ids,
                start_immediately,
                stashed,
            } => {
                let message = RestartMessage {
                    task_ids: task_ids.clone(),
                    start_immediately: *start_immediately,
                    stashed: *stashed,
                };
                Ok(Message::Restart(message))
            }
            SubCommand::Pause {
                task_ids,
                group,
                wait,
                all,
                children,
            } => {
                let message = PauseMessage {
                    task_ids: task_ids.clone(),
                    group: group.clone(),
                    wait: *wait,
                    all: *all,
                    children: *children,
                };
                Ok(Message::Pause(message))
            }
            SubCommand::Kill {
                task_ids,
                group,
                default,
                all,
                children,
            } => {
                let message = KillMessage {
                    task_ids: task_ids.clone(),
                    group: group.clone(),
                    default: *default,
                    all: *all,
                    children: *children,
                };
                Ok(Message::Kill(message))
            }

            SubCommand::Send { task_id, input } => {
                let message = SendMessage {
                    task_id: *task_id,
                    input: input.clone(),
                };
                Ok(Message::Send(message))
            }
            SubCommand::Edit { task_id, .. } => Ok(Message::EditRequest(*task_id)),
            SubCommand::Group { add, remove } => {
                let message = GroupMessage {
                    add: add.clone(),
                    remove: remove.clone(),
                };
                Ok(Message::Group(message))
            }
            SubCommand::Status { .. } => Ok(Message::Status),
            SubCommand::Log { task_ids, .. } => {
                let message = LogRequestMessage {
                    task_ids: task_ids.clone(),
                    send_logs: !self.settings.client.read_local_logs,
                };
                Ok(Message::Log(message))
            }
            SubCommand::Follow { task_id, err } => {
                let message = StreamRequestMessage {
                    task_id: task_id,
                    err: *err,
                };
                Ok(Message::StreamRequest(message))
            }
            SubCommand::Clean => Ok(Message::Clean),
            SubCommand::Reset { children } => Ok(Message::Reset(*children)),
            SubCommand::Shutdown => Ok(Message::DaemonShutdown),
            SubCommand::Parallel {
                parallel_tasks,
                group,
            } => {
                let message = ParallelMessage {
                    parallel_tasks: *parallel_tasks,
                    group: group.clone(),
                };
                Ok(Message::Parallel(message))
            }
            SubCommand::Completions { .. } => {
                Err(anyhow!("Completions have to be handled earlier"))
            }
        }
    }
}
