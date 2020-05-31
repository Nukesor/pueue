use ::anyhow::{anyhow, Context, Result};
use ::std::collections::HashMap;
use ::std::env::{current_dir, vars};

use ::pueue::message::*;
use ::pueue::settings::Settings;

use crate::cli::{Opt, SubCommand};

/// Convert the cli command into the message that's being sent to the server,
/// so it can be understood by the daemon.
pub fn get_message_from_opt(opt: &Opt, settings: &Settings) -> Result<Message> {
    match &opt.cmd {
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
        } => {
            let message = KillMessage {
                task_ids: task_ids.clone(),
                group: group.clone(),
                default: *default,
                all: *all,
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
                send_logs: !settings.client.read_local_logs,
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
        SubCommand::Clean => Ok(Message::Clean),
        SubCommand::Reset => Ok(Message::Reset),
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
        SubCommand::Completions { .. } => Err(anyhow!("Completions have to be handled earlier")),
    }
}
