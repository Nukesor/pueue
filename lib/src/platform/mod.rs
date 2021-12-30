/// MacOs specific stuff
#[cfg(target_vendor = "apple")]
pub mod apple;
/// Linux specific stuff
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub mod linux;
/// Windows specific stuff
#[cfg(target_os = "windows")]
pub mod windows;

// The next block is platform specific directory functions
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use self::linux::directories;

#[cfg(target_vendor = "apple")]
pub use self::apple::directories;

#[cfg(target_os = "windows")]
pub use self::windows::directories;
