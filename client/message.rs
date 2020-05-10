use ::anyhow::{anyhow, Context, Result};
use ::std::env::current_dir;

use ::pueue::message::*;
use ::pueue::settings::Settings;

use crate::cli::{Opt, SubCommand};

// Convert and pre-process the sub-command into a valid message
// that can be understood by the daemon.
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
            Ok(Message::Add(AddMessage {
                command: command.join(" "),
                path: cwd.to_string(),
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
        SubCommand::Start { task_ids } => Ok(Message::Start(task_ids.clone())),
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
        SubCommand::Pause { wait, task_ids } => {
            let message = PauseMessage {
                wait: *wait,
                task_ids: task_ids.clone(),
            };
            Ok(Message::Pause(message))
        }
        SubCommand::Kill { all, task_ids } => {
            let message = KillMessage {
                all: *all,
                task_ids: task_ids.clone(),
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
        SubCommand::Edit { task_id, path: _ } => Ok(Message::EditRequest(*task_id)),

        SubCommand::Status { json: _ } => Ok(Message::Status),
        SubCommand::Log { task_ids, json: _ } => {
            let message = LogRequestMessage {
                task_ids: task_ids.clone(),
                send_logs: !settings.client.read_local_logs,
            };
            Ok(Message::Log(message))
        }
        SubCommand::Show {
            task_id,
            follow,
            err,
        } => {
            let message = StreamRequestMessage {
                task_id: *task_id,
                follow: *follow,
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
        SubCommand::Completions {
            shell: _,
            output_directory: _,
        } => Err(anyhow!("Completions have to be handled earlier")),
    }
}
