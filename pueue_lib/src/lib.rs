#![doc = include_str!("../README.md")]

pub(crate) mod internal_prelude {
    #![allow(unused_imports)]
    pub use tracing::{debug, error, info, trace, warn};
}

pub mod error;
#[cfg(feature = "log")]
pub mod log;
pub mod message;
#[cfg(feature = "network")]
pub mod network;
#[cfg(feature = "network_blocking")]
pub mod network_blocking;
#[cfg(feature = "secret")]
pub mod secret;
#[cfg(feature = "settings")]
mod setting_defaults;
#[cfg(feature = "settings")]
pub mod settings;
pub mod state;
pub mod task;
#[cfg(feature = "tls")]
pub mod tls;

pub const PROTOCOL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use error::Error;
pub use message::{Request, Response};
#[cfg(all(feature = "client", feature = "network"))]
pub use network::client::Client;
#[cfg(all(feature = "client", feature = "network_blocking"))]
pub use network_blocking::client::BlockingClient;
#[cfg(feature = "settings")]
pub use settings::Settings;
pub use state::{Group, GroupStatus, State};
pub use task::{Task, TaskResult, TaskStatus};

pub mod prelude {
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
