use ::anyhow::{anyhow, Result};
use ::structopt::StructOpt;
use ::std::path::PathBuf;

use ::pueue::communication::message::*;

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    /// Queue a task for execution
    Add {
        /// The command that should be added
        #[structopt(value_delimiter = " ")]
        command: Vec<String>,

        /// Start the task immediately
        #[structopt(name = "start", short, long)]
        start_immediately: bool,
    },
    /// Wake the daemon from it's paused state, including continuing all paused tasks.
    /// Does nothing if the daemon isn't paused.
    Start {
        /// Enforce starting these tasks.
        /// Doesn't affect the daemon or any other tasks.
        #[structopt(short, long)]
        task_ids: Option<Vec<i32>>,
    },
    /// Display the current status of all tasks
    Status,
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Pueue client",
    about = "Interact with the Pueue daemon",
    author = "Arne Beer <contact@arne.beer>",
    version = "0.1",
)]
pub struct Opt {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Optional custom config path
    #[structopt(name = "config", parse(from_os_str))]
    pub config_path: Option<PathBuf>,

    #[structopt(subcommand)]
    pub cmd: SubCommand,
}


pub fn get_message_from_opt(opt: &Opt) -> Result<Message> {
    match &opt.cmd {
        SubCommand::Add{command, start_immediately}  => handle_add(command.clone(), *start_immediately),
        SubCommand::Status => Ok(Message::Status),
        _ => Err(anyhow!("Failed to interpret command. Please use --help")),
    }

}


fn handle_add(mut command: Vec<String>, start: bool) -> Result<Message> {
    // Unwrap because it's required
    println!("{:?}", command);
    Ok(Message::Add(AddMessage {
        command: command.remove(0),
        arguments: command,
        path: String::from("/"),
        start_immediately: start
    }))
}
