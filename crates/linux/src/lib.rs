#![cfg(target_os = "linux")]

mod dbus;
mod icon;
mod tray;

use gpui_tray_core::platform_trait::PlatformTray;
use gpui_tray_core::Result;

// Re-export for internal use by gpui-tray manager
pub use tray::{set_dispatcher_app, clear_dispatcher_app};

pub fn create() -> Result<Box<dyn PlatformTray>> {
    tray::create()
}

