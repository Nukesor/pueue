use clap::{App, Arg, SubCommand};

use crate::communication::message::*;

pub fn handle_cli() -> Message {
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

    Message::Add(AddMessage {
        command: matches.value_of("command").unwrap().to_string(),
        path: String::from("/"),
    })
}
