#[cfg(unix)]
mod internal_prelude {
    pub use color_eyre::{
        eyre::{bail, eyre, WrapErr},
        Result,
    };
}

#[cfg(unix)]
mod helper;

#[cfg(unix)]
mod client;

#[cfg(unix)]
mod daemon;
