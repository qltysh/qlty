#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub mod platform {
    #[cfg(target_os = "windows")]
    pub use super::windows::RubyWindows as Ruby;

    #[cfg(target_os = "linux")]
    pub use super::linux::RubyLinux as Ruby;

    #[cfg(target_os = "macos")]
    pub use super::macos::RubyMacos as Ruby;
}
