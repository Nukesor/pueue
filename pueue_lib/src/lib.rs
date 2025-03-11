#![doc = include_str!("../README.md")]

pub(crate) mod internal_prelude {
    #![allow(unused_imports)]
    pub use tracing::{debug, error, info, trace, warn};
}

/// A reference implementation of a simple client that you may use.
/// En/disable via the `client` feature.
#[cfg(feature = "client")]
pub mod client;
/// Pueue-lib errors.
pub mod error;
/// Helper classes to read and write log files of Pueue's tasks.
#[cfg(feature = "log")]
pub mod log;
/// This contains the the [`Request`] and [`Response`]  enums and
/// all their structs used to communicate with the daemon or client.
pub mod message;
#[cfg(feature = "network")]
pub mod network;
/// This module contains all platform unspecific default values and helper functions for working
/// with our setting representation.
#[cfg(feature = "settings")]
mod setting_defaults;
/// Pueue's configuration representation.
#[cfg(feature = "settings")]
pub mod settings;
/// The representation of all [`Task`]s and [`Group`]s of the daemon.
pub mod state;
/// Everything regarding Pueue's task
pub mod task;

pub const PROTOCOL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "client")]
pub use client::Client;
pub use error::Error;
pub use message::{Request, Response};
#[cfg(feature = "network")]
pub use network::protocol::{receive_request, receive_response, send_request, send_response};
#[cfg(feature = "settings")]
pub use settings::Settings;
pub use state::{Group, GroupStatus, State};
pub use task::{Task, TaskResult, TaskStatus};

pub mod prelude {
    #[cfg(feature = "client")]
    pub use super::client::Client;
    pub use super::error::Error;
    pub use super::message::{Request, Response};
    #[cfg(feature = "network")]
    pub use super::network::protocol::{
        receive_request, receive_response, send_request, send_response,
    };
    #[cfg(feature = "settings")]
    pub use super::settings::Settings;
    pub use super::state::{Group, GroupStatus, State};
    pub use super::task::{Task, TaskResult, TaskStatus};
}
