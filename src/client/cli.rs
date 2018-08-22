use clap::{App, Arg, SubCommand};
use serde_json;

use communication::message::*;

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
        ).get_matches();

    let add = AddMessage {
        command: matches.value_of("command").unwrap().to_string(),
        path: String::from("/"),
    };

    let message = Message {
        message_type: MessageTypes::Add,
        payload: serde_json::to_string(&add).unwrap(),
        add: Some(add),
    };

    message
}
