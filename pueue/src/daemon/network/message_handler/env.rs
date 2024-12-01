use pueue_lib::{network::message::*, settings::Settings, state::SharedState};

use crate::{
    daemon::{network::message_handler::ok_or_failure_message, state_helper::save_state},
    ok_or_save_state_failure,
};

/// Invoked on `pueue env`.
/// Manage environment variables for tasks.
/// - Set environment variables
/// - Unset environment variables
pub fn env(settings: &Settings, state: &SharedState, message: EnvMessage) -> Message {
    let mut state = state.lock().unwrap();

    let message = match message {
        EnvMessage::Set {
            task_id,
            key,
            value,
        } => {
            let Some(task) = state.tasks.get_mut(&task_id) else {
                return create_failure_message(format!("No task with id {task_id}"));
            };

            if !(task.is_queued() || task.is_stashed()) {
                return create_failure_message("You can only edit stashed or queued tasks");
            }

            task.envs.insert(key, value);

            create_success_message("Environment variable set.")
        }
        EnvMessage::Unset { task_id, key } => {
            let Some(task) = state.tasks.get_mut(&task_id) else {
                return create_failure_message(format!("No task with id {task_id}"));
            };

            if !(task.is_queued() || task.is_stashed()) {
                return create_failure_message("You can only edit stashed or queued tasks");
            }

            match task.envs.remove(&key) {
                Some(_) => create_success_message("Environment variable unset."),
                None => create_failure_message(format!(
                    "No environment variable with key '{key}' found."
                )),
            }
        }
    };

    // Save the state if there were any changes.
    if let Message::Success(_) = message {
        ok_or_save_state_failure!(save_state(&state, settings));
    }

    message
}
