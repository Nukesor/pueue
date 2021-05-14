use std::collections::BTreeMap;

use pueue_lib::network::message::TaskLogMessage;
use pueue_lib::settings::Settings;

pub fn print_log_json(task_logs: BTreeMap<usize, TaskLogMessage>, settings: &Settings) {
    println!("{}", serde_json::to_string(&task_logs).unwrap());
    return;
}
