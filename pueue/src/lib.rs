// This lint is generating way too many false-positives.
// Ignore it for now.
#![allow(clippy::assigning_clones)]
#![doc = include_str!("../README.md")]

pub(crate) mod prelude {
    #[allow(unused_imports)]
    pub(crate) use tracing::{debug, error, info, trace, warn};

    pub(crate) use crate::errors::*;
}

pub(crate) mod errors {
    pub use color_eyre::eyre::{WrapErr, bail};
    pub use color_eyre::Result;
}

pub mod client;
pub mod daemon;
