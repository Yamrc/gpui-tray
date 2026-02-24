#![cfg(target_os = "linux")]

mod dbus;
mod icon;
mod menu;
mod sni;
mod tray;

use gpui_tray_core::{PlatformTray, Result};
use log::debug;

pub use dbus::{TrayEventDispatcher, set_dispatcher};
pub use tray::LinuxTray;

pub fn create() -> Result<Box<dyn PlatformTray>> {
    tray::create()
}
