// This lint is generating way too many false-positives.
// Ignore it for now.
#![allow(clippy::assigning_clones)]
#![doc = include_str!("../README.md")]

pub(crate) mod internal_prelude {
    #[allow(unused_imports)]
    pub(crate) use tracing::{debug, error, info, trace, warn};

    pub(crate) use crate::errors::*;
}

pub(crate) mod errors {
    pub use color_eyre::Result;
    #[allow(unused_imports)]
    pub use color_eyre::eyre::{WrapErr, bail, eyre};
}

pub mod client;
pub mod daemon;
/// Shared module for internal logic!
/// Contains helper to spawn shell commands and examine and interact with processes.
pub mod process_helper;
pub mod tracing;
