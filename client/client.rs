use std::env::{current_dir, vars};
use std::io::{self, Write};
use std::{borrow::Cow, collections::HashMap};

use anyhow::{bail, Context, Result};
use clap::crate_version;
use colors::Colors;
use log::error;

use pueue_lib::network::message::*;
use pueue_lib::network::protocol::*;
use pueue_lib::network::secret::read_shared_secret;
use pueue_lib::settings::{Settings, PUEUE_DEFAULT_GROUP};

use crate::cli::{CliArguments, GroupSubcommand, SubCommand};
use crate::commands::edit::edit;
use crate::commands::get_state;
use crate::commands::local_follow::local_follow;
use crate::commands::restart::restart;
use crate::commands::wait::wait;
use crate::display::*;

/// This struct contains the base logic for the client.
/// The client is responsible for connecting to the daemon, sending instructions
/// and interpreting their responses.
///
/// Most commands are a simple ping-pong. However, some commands require a more complex
/// communication pattern, such as the `follow` command, which can read local files,
/// or the `edit` command, which needs to open an editor locally.
pub struct Client {
    subcommand: SubCommand,
    settings: Settings,
    colors: Colors,
    stream: GenericStream,
}

/// This is a small helper which either returns a given group or the default group.
pub fn group_or_default(group: &Option<String>) -> String {
    group
        .clone()
        .unwrap_or_else(|| PUEUE_DEFAULT_GROUP.to_string())
}

/// This is a small helper which determines the selection depending on given commandline
/// parameters.
/// If no parameters are given, it returns to the default group.
pub fn selection_from_params(
    all: bool,
    group: &Option<String>,
    task_ids: &[usize],
) -> TaskSelection {
    if all {
        TaskSelection::All
    } else if let Some(group) = group {
        TaskSelection::Group(group.clone())
    } else if !task_ids.is_empty() {
        TaskSelection::TaskIds(task_ids.to_owned())
    } else {
        TaskSelection::Group(PUEUE_DEFAULT_GROUP.into())
    }
}

impl Client {
    /// Connect to the daemon, authorize via secret and return a new initialized Client.
    pub async fn new(settings: Settings, opt: CliArguments) -> Result<Self> {
        // Connect to daemon and get stream used for communication.
        let mut stream = get_client_stream(&settings.shared).await?;

        // Next we do a handshake with the daemon
        // 1. Client sends the secret to the daemon.
        // 2. If successful, the daemon responds with their version.
        let secret = read_shared_secret(&settings.shared.shared_secret_path())?;
        send_bytes(&secret, &mut stream).await?;
        let version_bytes = receive_bytes(&mut stream)
            .await
            .context("Failed sending secret during handshake with daemon.")?;

        if version_bytes.is_empty() {
            bail!("Daemon went away after sending secret. Did you use the correct secret?")
        }

        // Check if we got valid utf8. Invalid utf8 should never happen and probably
        let version = match String::from_utf8(version_bytes) {
            Ok(version) => version,
            Err(_) => {
                bail!("Daemon went away after sending secret. Did you use the correct secret?")
            }
        };

        // Info if the daemon runs a different version.
        // Backward compatibility should work, but some features might not work as expected.
        if version != crate_version!() {
            println!(
                "Different daemon version detected '{}'. Consider restarting the daemon.",
                version
            );
        }

        let colors = Colors::new(&settings);

        // If no subcommand is given, we default to the `status` subcommand without any arguments.
        let subcommand = if let Some(subcommand) = opt.cmd {
            subcommand
        } else {
            SubCommand::Status {
                json: false,
                group: None,
            }
        };

        Ok(Client {
            settings,
            colors,
            stream,
            subcommand,
        })
    }

    /// This is the function where the actual communication and logic starts.
    /// At this point everything is initialized, the connection is up and
    /// we can finally start doing stuff.
    ///
    /// The command handling is splitted into "simple" and "complex" commands.
    pub async fn start(&mut self) -> Result<()> {
        // Return early, if the command has already been handled.
        if self.handle_complex_command().await? {
            return Ok(());
        }

        // The handling of "generic" commands is encapsulated in this function.
        self.handle_simple_command().await?;

        Ok(())
    }

    /// Handle all complex client-side functionalities.
    /// Complex functionalities need some special handling and are contained
    /// in their own functions with their own communication code.
    /// Such functionalities includes reading local filestand sending multiple messages.
    ///
    /// Returns `Ok(true)`, if the current command has been handled by this function.
    /// This indicates that the client can now shut down.
    /// If `Ok(false)` is returned, the client will continue and handle the Subcommand in the
    /// [handle_simple_command] function.
    async fn handle_complex_command(&mut self) -> Result<bool> {
        // This match handles all "complex" commands.
        match &self.subcommand {
            SubCommand::Reset { force, .. } => {
                let state = get_state(&mut self.stream).await?;
                let running_tasks = state
                    .tasks
                    .iter()
                    .filter_map(|(id, task)| if task.is_running() { Some(*id) } else { None })
                    .collect::<Vec<_>>();

                if !running_tasks.is_empty() && !force {
                    self.handle_user_confirmation("remove running tasks", &running_tasks)?;
                }

                // Let handle_simple_command to handle `reset` after getting user permission to kill
                // running tasks
                Ok(false)
            }

            SubCommand::Edit { task_id, path } => {
                let message = edit(&mut self.stream, *task_id, *path).await?;
                self.handle_response(message);
                Ok(true)
            }
            SubCommand::Wait {
                task_ids,
                group,
                all,
                quiet,
            } => {
                let group = group_or_default(group);
                wait(
                    &mut self.stream,
                    task_ids,
                    &group,
                    *all,
                    *quiet,
                    &self.colors,
                )
                .await?;
                Ok(true)
            }
            SubCommand::Restart {
                task_ids,
                all_failed,
                failed_in_group,
                start_immediately,
                stashed,
                in_place,
                not_in_place,
                edit,
                edit_path,
            } => {
                // `not_in_place` superseeds both other configs
                let in_place =
                    (self.settings.client.restart_in_place || *in_place) && !*not_in_place;
                restart(
                    &mut self.stream,
                    task_ids.clone(),
                    *all_failed,
                    failed_in_group.clone(),
                    *start_immediately,
                    *stashed,
                    in_place,
                    *edit,
                    *edit_path,
                )
                .await?;
                Ok(true)
            }

            SubCommand::Follow { task_id, err } => {
                // Simple log output follows for local logs don't need any communication with the daemon.
                // Thereby we handle this separately over here.
                if self.settings.client.read_local_logs {
                    local_follow(
                        &mut self.stream,
                        &self.settings.shared.pueue_directory(),
                        task_id,
                        *err,
                    )
                    .await?;
                    return Ok(true);
                }
                Ok(false)
            }

            _ => Ok(false),
        }
    }

    /// Handle logic that's super generic on the client-side.
    /// This (almost) always follows a singular ping-pong pattern.
    /// One message to the daemon, one response, Done.
    ///
    /// The only exception is streaming of log output.
    /// In that case, we send one request and contine receiving until the stream shuts down.
    async fn handle_simple_command(&mut self) -> Result<()> {
        // Create the message that should be sent to the daemon
        // depending on the given commandline options.
        let message = self.get_message_from_opt()?;

        // Create the message payload and send it to the daemon.
        send_message(message, &mut self.stream).await?;

        // Check if we can receive the response from the daemon
        let mut response = receive_message(&mut self.stream).await?;

        // Check if we can receive the response from the daemon
        while self.handle_response(response) {
            response = receive_message(&mut self.stream).await?;
        }

        Ok(())
    }

    /// Most returned messages can be handled in a generic fashion.
    /// However, some commands require continuous receiving of messages (streaming).
    ///
    /// If this function returns `Ok(true)`, the parent function will continue to receive
    /// and handle messages from the daemon. Otherwise the client will simply exit.
    fn handle_response(&self, message: Message) -> bool {
        match message {
            Message::Success(text) => print_success(&self.colors, &text),
            Message::Failure(text) => {
                print_error(&self.colors, &text);
                std::process::exit(1);
            }
            Message::StatusResponse(state) => {
                print_state(*state, &self.subcommand, &self.colors, &self.settings)
            }
            Message::LogResponse(task_logs) => {
                print_logs(task_logs, &self.subcommand, &self.colors, &self.settings)
            }
            Message::GroupResponse(groups) => print_groups(groups, &self.colors),
            Message::Stream(text) => {
                print!("{}", text);
                io::stdout().flush().unwrap();
                return true;
            }
            _ => error!("Received unhandled response message"),
        };

        false
    }

    /// Prints a warning and prompt for given action and tasks.
    /// Returns `Ok(())` if the action was confirmed.
    fn handle_user_confirmation(&self, action: &str, task_ids: &[usize]) -> Result<()> {
        // printing warning and prompt
        println!(
            "You are trying to {}: {}",
            action,
            task_ids
                .iter()
                .map(|t| format!("task{}", t.to_string()))
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut input = String::new();

        loop {
            print!("Do you want to continue [Y/n]: ");
            io::stdout().flush().unwrap();
            input.clear();
            io::stdin().read_line(&mut input)?;

            match input.chars().next().unwrap() {
                'N' | 'n' => {
                    println!("Aborted!");
                    std::process::exit(1);
                }
                '\n' | 'Y' | 'y' => {
                    break;
                }
                _ => {
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Convert the cli command into the message that's being sent to the server,
    /// so it can be understood by the daemon.
    fn get_message_from_opt(&self) -> Result<Message> {
        match &self.subcommand {
            SubCommand::Add {
                command,
                working_directory: cwd,
                escape,
                start_immediately,
                stashed,
                group,
                delay_until,
                dependencies,
                label,
                print_task_id,
            } => {
                let cwd_pathbuf = if let Some(cwd) = cwd {
                    cwd.clone()
                } else {
                    current_dir()?
                };

                let path = cwd_pathbuf
                    .to_str()
                    .context("Cannot parse current working directory (Invalid utf8?)")?
                    .to_string();

                let mut envs = HashMap::new();
                // Save all environment variables for later injection into the started task
                for (key, value) in vars() {
                    envs.insert(key, value);
                }

                // Escape any special shell characters in all strings before we concatenated them
                // to a single string.
                let command: Vec<String> = if *escape {
                    command
                        .iter()
                        .map(|parameter| shell_escape::escape(Cow::from(parameter)).into_owned())
                        .collect()
                } else {
                    command.clone()
                };

                let group = group_or_default(group);
                Ok(Message::Add(AddMessage {
                    command: command.join(" "),
                    path,
                    envs,
                    start_immediately: *start_immediately,
                    stashed: *stashed,
                    group,
                    enqueue_at: *delay_until,
                    dependencies: dependencies.to_vec(),
                    label: label.clone(),
                    print_task_id: *print_task_id,
                }))
            }
            SubCommand::Remove { task_ids } => {
                if self.settings.client.show_confirmation_questions {
                    self.handle_user_confirmation("remove", task_ids)?;
                }
                Ok(Message::Remove(task_ids.clone()))
            }
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
                let selection = selection_from_params(*all, group, task_ids);
                let message = StartMessage {
                    tasks: selection,
                    children: *children,
                };
                Ok(Message::Start(message))
            }
            SubCommand::Pause {
                task_ids,
                group,
                wait,
                all,
                children,
            } => {
                let selection = selection_from_params(*all, group, task_ids);
                let message = PauseMessage {
                    tasks: selection,
                    wait: *wait,
                    children: *children,
                };
                Ok(Message::Pause(message))
            }
            SubCommand::Kill {
                task_ids,
                group,
                all,
                children,
                signal,
            } => {
                if self.settings.client.show_confirmation_questions {
                    self.handle_user_confirmation("kill", task_ids)?;
                }
                let selection = selection_from_params(*all, group, task_ids);
                let message = KillMessage {
                    tasks: selection,
                    children: *children,
                    signal: signal.clone(),
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
            SubCommand::Group { group_cmd } => {
                if let Some(cmd) = group_cmd {
                    match cmd {
                        GroupSubcommand::Add { name, parallel } => Ok(Message::Group(GroupMessage::Add(name.to_owned(), parallel.to_owned()))),
                        GroupSubcommand::Remove { name } => Ok(Message::Group(GroupMessage::Remove(name.to_owned())))
                    }
                } else {
                    Ok(Message::Group(GroupMessage::List))
                }
            }
            SubCommand::Status { .. } => Ok(Message::Status),
            SubCommand::Log {
                task_ids,
                lines,
                full,
                json,
            } => {
                let lines = determine_log_line_amount(*full, lines, *json, task_ids.len());

                let message = LogRequestMessage {
                    task_ids: task_ids.clone(),
                    send_logs: !self.settings.client.read_local_logs,
                    lines,
                };
                Ok(Message::Log(message))
            }
            SubCommand::Follow { task_id, err } => {
                let message = StreamRequestMessage {
                    task_id: *task_id,
                    err: *err,
                };
                Ok(Message::StreamRequest(message))
            }
            SubCommand::Clean { successful_only } => {
                let message = CleanMessage {
                    successful_only: *successful_only,
                };

                Ok(Message::Clean(message))
            }
            SubCommand::Reset { children, force } => {
                if self.settings.client.show_confirmation_questions && !force {
                    self.handle_user_confirmation("reset", &Vec::new())?;
                }

                let message = ResetMessage {
                    children: *children,
                };
                Ok(Message::Reset(message))
            }
            SubCommand::Shutdown => Ok(Message::DaemonShutdown(Shutdown::Graceful)),
            SubCommand::Parallel {
                parallel_tasks,
                group,
            } => {
                let group = group_or_default(group);
                let message = ParallelMessage {
                    parallel_tasks: *parallel_tasks,
                    group,
                };
                Ok(Message::Parallel(message))
            }
            SubCommand::Completions { .. } => bail!("Completions have to be handled earlier"),
            SubCommand::Restart { .. } => bail!("Restarts have to be handled earlier"),
            SubCommand::Edit { .. } => bail!("Edits have to be handled earlier"),
            SubCommand::Wait { .. } => bail!("Wait has to be handled earlier"),
        }
    }
}
