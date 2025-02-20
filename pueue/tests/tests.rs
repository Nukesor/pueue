#[cfg(unix)]
mod internal_prelude {
    pub use color_eyre::{
        Result,
        eyre::{WrapErr, bail, eyre},
    };
    pub use tracing::debug;
}

#[cfg(unix)]
mod helper;

#[cfg(unix)]
mod client;

#[cfg(unix)]
mod daemon;
