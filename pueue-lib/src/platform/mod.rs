/// Linux specific stuff
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub mod linux;
/// MacOs specific stuff
#[cfg(target_os = "macos")]
pub mod macos;
/// Windows specific stuff
#[cfg(target_os = "windows")]
pub mod windows;

// The next block is platform specific directory functions
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use self::linux::directories;

#[cfg(target_os = "macos")]
pub use self::macos::directories;

#[cfg(target_os = "windows")]
pub use self::windows::directories;
