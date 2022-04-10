//! Pueue-lib is a shared library used by the `pueue` and `pueued` binary.
//!
//! It contains common components such as:
//!
//! - Everything about the [Task](task::Task), [TaskResult](task::TaskResult) etc.
//! - The [State](state::State), which represents the current state of the daemon.
//! - Network code. Everything you need to communicate with the daemon.
//! - Other helper code and structs.
//!
//! Pueue-lib is a stand-alone crate, so it can be used by third-party applications to either
//! manipulate or monitor the daemon or to simply write your own front-end for the daemon.

/// Contains helper for command aliasing. This will most likely be not interesting for you.
pub mod aliasing;
/// Pueue lib's own Error implementation.
pub mod error;
/// Helper classes to read and write log files of Pueue's tasks.
pub mod log;
pub mod network;
/// Platform specific code, mainly used to get platform specific working directories.
mod platform;
/// This module contains all platform unspecific default values and helper functions for working
/// with our setting representation.
mod setting_defaults;
/// Pueue's representation of configuration and their default settings.
pub mod settings;
/// The main struct used to represent the daemon's current state.
pub mod state;
/// Everything regarding Pueue's task
pub mod task;
