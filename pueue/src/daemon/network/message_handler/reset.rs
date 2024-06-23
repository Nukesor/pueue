use pueue_lib::state::SharedState;
use pueue_lib::{network::message::*, settings::Settings};

use crate::daemon::process_handler;

/// Invoked when calling `pueue reset`.
/// Kill all children by using the `kill` function.
/// Set the full_reset flag, which will prevent new tasks from being spawned.
pub fn reset(settings: &Settings, state: &SharedState) -> Message {
    let mut state = state.lock().unwrap();
    state.full_reset = true;
    process_handler::kill::kill(settings, &mut state, TaskSelection::All, false, None);
    create_success_message("Everything is being reset right now.")
}
