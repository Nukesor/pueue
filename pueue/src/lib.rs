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
    #[allow(unused_imports)]
    pub use color_eyre::eyre::{bail, eyre, WrapErr};
    pub use color_eyre::Result;
}

pub mod client;
pub mod daemon;
/// Shared module for internal logic!
/// Contains helper to spawn shell commands and examine and interact with processes.
pub mod process_helper;
pub mod tracing;
