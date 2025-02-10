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
    pub use color_eyre::eyre::{bail, WrapErr};
    pub use color_eyre::Result;
}

pub mod client;
pub mod daemon;
pub mod tracing;
