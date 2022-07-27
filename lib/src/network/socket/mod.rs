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
