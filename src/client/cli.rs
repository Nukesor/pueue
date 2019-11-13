use ::anyhow::{anyhow, Result};
use clap::{App, Arg};

use crate::communication::message::*;

pub fn handle_cli() -> Result<Message> {
    let matches = App::new("Pueue client")
        .version("0.1")
        .author("Arne Beer <contact@arne.beer>")
        .about("The client application to communicate and manipulate the pueue daemon")
        .arg(
            Arg::with_name("command")
                .help("Command to execute")
                .required(true)
                .index(1),
        )
        .get_matches();

    let command = matches
        .value_of("command")
        .ok_or(anyhow!("You need to specify a command"))?;

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
