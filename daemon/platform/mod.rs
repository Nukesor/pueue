#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod linux;
#[cfg(any(target_os = "macos"))]
mod macos;
#[cfg(any(target_os = "windows"))]
mod windows;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use self::linux::process_helper;

#[cfg(target_os = "macos")]
pub use self::macos::process_helper;

#[cfg(target_os = "windows")]
pub use self::windows::process_helper;
