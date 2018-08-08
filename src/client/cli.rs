use clap::{App, Arg, ArgMatches, SubCommand};

pub fn get_app<'a>() -> ArgMatches<'a> {
    let matches = App::new("Pueue client")
        .version("0.1")
        .author("Arne Beer <contact@arne.beer>")
        .about("The client application to communicate and manipulate the pueue daemon")
        .arg(Arg::with_name("command")
             .help("Command to execute")
             .required(true)
             .index(1))
        .get_matches();

    return matches;
}
