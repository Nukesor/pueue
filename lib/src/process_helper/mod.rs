// Unix specific process handling
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use self::linux::*;

// Apple specific process handling
#[cfg(any(target_vendor = "apple"))]
mod apple;
#[cfg(target_vendor = "apple")]
pub use self::apple::*;

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
