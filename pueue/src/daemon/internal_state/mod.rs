use std::sync::{Arc, Mutex};

use state::InternalState;

/// A helper newtype struct, which implements convenience methods for our child process management
/// datastructure.
pub mod children;
/// The main struct used to represent the daemon's current state.
pub mod state;

pub type SharedState = Arc<Mutex<InternalState>>;
