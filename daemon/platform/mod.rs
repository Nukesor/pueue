#[cfg(any(target_vendor = "apple"))]
mod apple;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod linux;
#[cfg(any(target_os = "windows"))]
mod windows;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use self::linux::process_helper;

#[cfg(target_vendor = "apple")]
pub use self::apple::process_helper;

#[cfg(target_os = "windows")]
pub use self::windows::process_helper;
