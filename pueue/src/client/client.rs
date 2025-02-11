use std::io::{self, stdout, Write};

use clap::crate_version;
use crossterm::tty::IsTty;
use pueue_lib::{
    network::{message::*, protocol::*, secret::read_shared_secret},
    settings::Settings,
    Error,
};
use serde::Serialize;

use super::commands::*;
use crate::{
    client::{
        cli::{ColorChoice, SubCommand},
        display::*,
    },
    internal_prelude::*,
};

/// This struct contains the base logic for the client.
/// The client is responsible for connecting to the daemon, sending instructions
/// and interpreting their responses.
///
/// Most commands are a simple ping-pong. However, some commands require a more complex
/// communication pattern, such as the `follow` command, which can read local files,
/// or the `edit` command, which needs to open an editor.
pub struct Client {
    pub settings: Settings,
    pub style: OutputStyle,
    pub stream: GenericStream,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("settings", &self.settings)
            .field("style", &self.style)
            .field("stream", &"GenericStream<not_debuggable>")
            .finish()
    }
}

impl Client {
    /// Initialize a new client.
    /// This includes establishing a connection to the daemon:
    ///     - Connect to the daemon.
    ///     - Authorize via secret.
    ///     - Check versions incompatibilities.
    pub async fn new(
        settings: Settings,
        show_version_warning: bool,
        color: &ColorChoice,
    ) -> Result<Self> {
        // Connect to daemon and get stream used for communication.
        let mut stream = get_client_stream(&settings.shared)
            .await
            .context("Failed to initialize stream.")?;

        // Next we do a handshake with the daemon
        // 1. Client sends the secret to the daemon.
        // 2. If successful, the daemon responds with their version.
        let secret = read_shared_secret(&settings.shared.shared_secret_path())?;
        send_bytes(&secret, &mut stream)
            .await
            .context("Failed to send secret.")?;

        // Receive and parse the response. We expect the daemon's version as UTF-8.
        let version_bytes = receive_bytes(&mut stream)
            .await
            .context("Failed to receive version during handshake with daemon.")?;
        if version_bytes.is_empty() {
            bail!("Daemon went away after sending secret. Did you use the correct secret?")
        }
        let version = match String::from_utf8(version_bytes) {
            Ok(version) => version,
            Err(_) => {
                bail!("Daemon sent invalid UTF-8. Did you use the correct secret?")
            }
        };

        // Info if the daemon runs a different version.
        // Backward compatibility should work, but some features might not work as expected.
        if version != crate_version!() && show_version_warning {
            warn!("Different daemon version detected '{version}'. Consider restarting the daemon.");
        }

        // Determine whether we should color/style our output or not.
        // The user can explicitly disable/enable this, otherwise we check whether we are on a TTY.
        let style_enabled = match color {
            ColorChoice::Auto => stdout().is_tty(),
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        };
        let style = OutputStyle::new(&settings, style_enabled);

        Ok(Client {
            settings,
            style,
            stream,
        })
    }

    /// Convenience function to get a mutable handle on the client's stream.
    pub fn stream(&mut self) -> &mut GenericStream {
        &mut self.stream
    }

    /// Convenience wrapper around [`pueue_lib::send_request`] to directly send [`Request`]s.
    pub async fn send_request<T>(&mut self, message: T) -> Result<(), Error>
    where
        T: Into<Request>,
        T: Serialize + std::fmt::Debug,
    {
        send_message::<_, Request>(message, &mut self.stream).await
    }

    /// Convenience wrapper that wraps `receive_message` for [`Response`]s
    pub async fn receive_response(&mut self) -> Result<Response, Error> {
        receive_message::<Response>(&mut self.stream).await
    }
}

/// Handle all commands.
///
/// This is the core entry point of the pueue client.
/// Based on the subcommand, the respective function in the [`super::commands`] module is
/// called.
pub async fn handle_command(client: &mut Client, subcommand: SubCommand) -> Result<()> {
    trace!(message = "Handling command", subcommand = ?subcommand);

    match subcommand {
        SubCommand::Add {
            command,
            working_directory,
            escape,
            start_immediately,
            stashed,
            group,
            delay_until,
            dependencies,
            priority,
            label,
            print_task_id,
            follow,
        } => {
            add_task(
                client,
                command,
                working_directory,
                escape,
                start_immediately,
                stashed,
                group,
                delay_until,
                dependencies,
                priority,
                label,
                print_task_id,
                follow,
            )
            .await
        }
        SubCommand::Clean {
            successful_only,
            group,
        } => clean(client, group, successful_only).await,
        SubCommand::Edit { task_ids } => edit(client, task_ids).await,
        SubCommand::Enqueue {
            task_ids,
            group,
            all,
            delay_until,
        } => enqueue(client, task_ids, group, all, delay_until).await,
        SubCommand::Env { cmd } => env(client, cmd).await,
        SubCommand::Follow { task_id, lines } => follow(client, task_id, lines).await,
        SubCommand::FormatStatus { group } => format_state(client, group).await,
        SubCommand::Group { cmd, json } => group(client, cmd, json).await,
        SubCommand::Kill {
            task_ids,
            group,
            all,
            signal,
        } => kill(client, task_ids, group, all, signal).await,
        SubCommand::Log {
            task_ids,
            group,
            all,
            json,
            lines,
            full,
        } => print_logs(client, task_ids, group, all, json, lines, full).await,
        SubCommand::Parallel {
            parallel_tasks,
            group,
        } => parallel(client, parallel_tasks, group).await,
        SubCommand::Pause {
            task_ids,
            group,
            all,
            wait,
        } => pause(client, task_ids, group, all, wait).await,
        SubCommand::Remove { task_ids } => remove(client, task_ids).await,
        SubCommand::Reset { force, groups } => reset(client, force, groups).await,
        SubCommand::Restart {
            task_ids,
            all_failed,
            failed_in_group,
            start_immediately,
            stashed,
            in_place,
            not_in_place,
            edit,
        } => {
            restart(
                client,
                task_ids,
                all_failed,
                failed_in_group,
                start_immediately,
                stashed,
                in_place,
                not_in_place,
                edit,
            )
            .await
        }
        SubCommand::Send { task_id, input } => send(client, task_id, input).await,
        SubCommand::Shutdown => shutdown(client).await,
        SubCommand::Stash {
            task_ids,
            group,
            all,
            delay_until,
        } => stash(client, task_ids, group, all, delay_until).await,
        SubCommand::Start {
            task_ids,
            group,
            all,
        } => start(client, task_ids, group, all).await,
        SubCommand::Status { query, json, group } => state(client, query, json, group).await,
        SubCommand::Switch {
            task_id_1,
            task_id_2,
        } => switch(client, task_id_1, task_id_2).await,
        SubCommand::Wait {
            task_ids,
            group,
            all,
            quiet,
            status,
        } => wait(client, task_ids, group, all, quiet, status).await,
        _ => bail!("unhandled WIP"),
    }
}

/// Prints a warning and prompt for a given action and tasks.
/// Returns `Ok(())` if the action was confirmed.
pub fn handle_user_confirmation(action: &str, task_ids: &[usize]) -> Result<()> {
    // printing warning and prompt
    let task_ids = task_ids
        .iter()
        .map(|t| format!("task{t}"))
        .collect::<Vec<String>>()
        .join(", ");
    eprintln!("You are trying to {action}: {task_ids}",);

    let mut input = String::new();

    loop {
        print!("Do you want to continue [Y/n]: ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;

        match input.chars().next().unwrap() {
            'N' | 'n' => {
                eprintln!("Aborted!");
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
