//! Subprocess handling is platform specific code.
//!
//! The submodules of this module represent the different implementations for
//! each supported platform.
//! Depending on the target, the respective platform is read and loaded into this scope.

use crate::network::message::Signal as InternalSignal;

// Unix specific process handling
// Shared between Linux and Apple
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::*;
#[cfg(unix)]
use command_group::Signal;

// Linux specific process support
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::process_exists;

// Apple specific process support
#[cfg(target_vendor = "apple")]
mod apple;
#[cfg(target_vendor = "apple")]
pub use self::apple::process_exists;

// Windows specific process handling
#[cfg(any(target_os = "windows"))]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

/// Pueue directly interacts with processes.
/// Since these interactions can vary depending on the current platform, this enum is introduced.
/// The intend is to keep any platform specific code out of the top level code.
/// Even if that implicates adding some layers of abstraction.
#[derive(Debug)]
pub enum ProcessAction {
    Pause,
    Resume,
}

impl From<&ProcessAction> for Signal {
    fn from(action: &ProcessAction) -> Self {
        match action {
            ProcessAction::Pause => Signal::SIGSTOP,
            ProcessAction::Resume => Signal::SIGCONT,
        }
    }
}

impl From<InternalSignal> for Signal {
    fn from(signal: InternalSignal) -> Self {
        match signal {
            InternalSignal::SigKill => Signal::SIGKILL,
            InternalSignal::SigInt => Signal::SIGINT,
            InternalSignal::SigTerm => Signal::SIGTERM,
            InternalSignal::SigCont => Signal::SIGCONT,
            InternalSignal::SigStop => Signal::SIGSTOP,
        }
    }
}
