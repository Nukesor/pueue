use std::collections::HashMap;

use pueue_lib::network::message::*;
use pueue_lib::settings::*;
use pueue_lib::state::PUEUE_DEFAULT_GROUP;

/// Create a AddMessage for a given command.
pub fn add_message(shared: &Shared, command: &str) -> AddMessage {
    AddMessage {
        command: command.into(),
        path: shared.pueue_directory().to_str().unwrap().to_string(),
        envs: HashMap::new(),
        start_immediately: false,
        stashed: false,
        group: PUEUE_DEFAULT_GROUP.into(),
        enqueue_at: None,
        dependencies: vec![],
        label: None,
        print_task_id: false,
    }
}
