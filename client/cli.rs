use ::anyhow::{anyhow, Result};
use clap::{App, Arg, SubCommand, ArgMatches};

use ::pueue::communication::message::*;

pub fn handle_cli() -> ArgMatches<'static> {
    let matches = App::new("Pueue client")
        .version("0.1")
        .author("Arne Beer <contact@arne.beer>")
        .about("The client application to communicate and manipulate the pueue daemon")
        .arg(Arg::with_name("verbose")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .subcommand(SubCommand::with_name("add")
            .about("Queue a task for execution")
            .arg(Arg::with_name("command")
                    .help("Command to execute")
                    .required(true)
                    .index(1))
        )
        // Status subcommand
        .subcommand(SubCommand::with_name("status")
            .about("Show the current status of the deamon.")
        )
        .get_matches();

    matches
}


pub fn get_message_from_matches(matches: &ArgMatches) -> Result<Message> {
    if let Some(ref matches) = matches.subcommand_matches("add") {
        return handle_add(matches);
    }

    if let Some(_) = matches.subcommand_matches("status") {
        return Ok(Message::Status);
    }

    Err(anyhow!("Failed to interpret command. Please use --help"))
}


fn handle_add(matches: &ArgMatches) -> Result<Message> {
    // Unwrap because it's required
    let command = matches.value_of("command").unwrap();

    let mut command: Vec<String> = command
        .to_string()
        .split(" ")
        .map(|x| x.to_string())
        .collect();

    Ok(Message::Add(AddMessage {
        command: command.remove(0),
        arguments: command,
        path: String::from("/"),
    }))
}
