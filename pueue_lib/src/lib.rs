#![doc = include_str!("../README.md")]

pub(crate) mod internal_prelude {
    #![allow(unused_imports)]
    pub use tracing::{debug, error, info, trace, warn};
}

/// Shared module for internal logic!
/// Contains helper for command aliasing.
pub mod aliasing;
/// A reference implementation of a simple client that you may use.
/// En/disable via the `client` feature.
#[cfg(feature = "client")]
pub mod client;
/// Pueue lib's own Error implementation.
pub mod error;
/// Formatting methods for several data types.
pub mod format;
/// Helper classes to read and write log files of Pueue's tasks.
pub mod log;
pub mod network;
/// This module contains all platform unspecific default values and helper functions for working
/// with our setting representation.
mod setting_defaults;
/// Pueue's configuration representation.
pub mod settings;
/// The representation of all [`Task`]s and [`Group`]s of the daemon.
pub mod state;
/// Everything regarding Pueue's task
pub mod task;

pub const PROTOCOL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "client")]
pub use client::Client;
pub use error::Error;
pub use network::{
    message::{Request, Response},
    protocol::{receive_request, receive_response, send_request, send_response},
};
pub use settings::Settings;
pub use state::{Group, GroupStatus, State};
pub use task::{Task, TaskResult, TaskStatus};

pub mod prelude {
    #[cfg(feature = "client")]
    pub use super::client::Client;
    pub use super::error::Error;
    pub use super::network::{
        message::{Request, Response},
        protocol::{receive_request, receive_response, send_request, send_response},
    };
    pub use super::settings::Settings;
    pub use super::state::{Group, GroupStatus, State};
    pub use super::task::{Task, TaskResult, TaskStatus};
}
