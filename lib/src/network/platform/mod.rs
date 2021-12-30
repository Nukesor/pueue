/// Shared unix stuff
#[cfg(not(target_os = "windows"))]
pub mod unix;
/// Windows specific stuff
#[cfg(target_os = "windows")]
pub mod windows;

/// Shared unix stuff for sockets
#[cfg(not(target_os = "windows"))]
pub use self::unix::socket;

/// Windows specific socket stuff
#[cfg(target_os = "windows")]
pub use self::windows::socket;
