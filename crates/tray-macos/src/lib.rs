//! macOS system tray implementation
//!
//! This crate provides macOS-specific implementation for system tray.
//! It is used internally by gpui-tray and should not be used directly.
//!
//! TODO: Implement macOS support.

pub mod state;
pub mod tray;

pub use state::MacosTrayState;
pub use tray::{MacosTray, MacosTrayConfig};
