#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod stub;

#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use stub::*;
#[cfg(target_os = "windows")]
pub use windows::*;
