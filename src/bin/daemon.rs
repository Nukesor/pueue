extern crate pueue;
extern crate daemonize;

use daemonize::{Daemonize};

fn main() {
    let daemonize = Daemonize::new()
        .pid_file("/tmp/pueue.pid") // Every method except `new` and `start`
        .chown_pid_file(true)      // is optional, see `Daemonize` documentation
        .working_directory("/tmp") // for default behaviour.
        .user("nobody")
        .group("daemon")
        .umask(0o777)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("{}", e),
    }
}
