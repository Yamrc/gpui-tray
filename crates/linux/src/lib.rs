#![cfg(target_os = "linux")]

use tray_core::Result;
use tray_core::platform_trait::PlatformTray;

mod dbus;
mod icon;
mod tray;

pub fn create() -> Result<Box<dyn PlatformTray>> {
    tray::create()
}
