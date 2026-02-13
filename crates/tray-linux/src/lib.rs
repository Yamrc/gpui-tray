//! Linux system tray implementation
//!
//! This crate provides Linux-specific implementation for system tray.
//! It is used internally by gpui-tray and should not be used directly.
//!
//! TODO: Implement linux support.

pub mod state;
pub mod tray;

pub use state::LinuxTrayState;
pub use tray::{LinuxTray, LinuxTrayConfig};
