#![cfg(target_os = "linux")]

use gpui_tray_core::Result;
use gpui_tray_core::platform_trait::PlatformTray;

mod dbus;
mod icon;
mod tray;

pub fn create() -> Result<Box<dyn PlatformTray>> {
    tray::create()
}
