//! Socket handling is platform specific code.
//!
//! The submodules of this module represent the different implementations for
//! each supported platform.
//! Depending on the target, the respective platform is read and loaded into this scope.

/// Shared unix stuff
#[cfg(not(target_os = "windows"))]
mod unix;
/// Shared unix stuff for sockets
#[cfg(not(target_os = "windows"))]
pub use self::unix::*;

/// Windows specific stuff
#[cfg(target_os = "windows")]
mod windows;
/// Windows specific socket stuff
#[cfg(target_os = "windows")]
pub use self::windows::*;
