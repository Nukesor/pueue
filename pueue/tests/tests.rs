#[cfg(unix)]
mod internal_prelude {
    pub use color_eyre::{
        eyre::{bail, eyre, WrapErr},
        Result,
    };
    pub use tracing::debug;
}

#[cfg(unix)]
mod helper;

#[cfg(unix)]
mod client;

#[cfg(unix)]
mod daemon;
