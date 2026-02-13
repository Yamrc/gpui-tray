//! Windows system tray implementation
//!
//! This crate provides Windows-specific implementation for system tray.
//! It is used internally by gpui-tray and should not be used directly.

pub mod state;
pub mod tray;
mod util;
mod window;

pub use state::WindowsTrayState;
pub use tray::{WindowsTray, WindowsTrayConfig};
