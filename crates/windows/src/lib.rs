mod tray;
mod util;
mod window;

use gpui_tray_core::{PlatformTray, Result};
use log::debug;

pub use tray::WindowsTray;

pub fn create() -> Result<Box<dyn PlatformTray>> {
    debug!("Creating Windows tray implementation");
    Ok(Box::new(WindowsTray::new()))
}
