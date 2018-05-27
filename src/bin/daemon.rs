extern crate pueue;
extern crate daemonize;
extern crate users;

use daemonize::{Daemonize};
use pueue::settings::Settings;
use users::{get_user_by_uid, get_current_uid};


fn main() {
    let user = get_user_by_uid(get_current_uid()).unwrap();
    let settings = Settings::new().unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/pueue.pid") // Every method except `new` and `start`
        .chown_pid_file(true)      // is optional, see `Daemonize` documentation
        .working_directory("/tmp") // for default behaviour.
        .user(user.name())
        .group(settings.common.group_id)
        .umask(0o777)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("{}", e),
    }
}
