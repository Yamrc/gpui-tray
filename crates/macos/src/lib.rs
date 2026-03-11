#![cfg(target_os = "macos")]

use tray_core::Result;
use tray_core::platform_trait::PlatformTray;

pub fn create() -> Result<Box<dyn PlatformTray>> {
    Err(tray_core::Error::UnsupportedPlatform)
}
