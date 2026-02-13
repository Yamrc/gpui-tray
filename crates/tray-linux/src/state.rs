//! Global state management for Linux tray

use gpui::Global;

use crate::tray::{LinuxTray, LinuxTrayConfig};

/// Global state for Linux tray
pub struct LinuxTrayState {
    tray: Option<LinuxTray>,
}

impl LinuxTrayState {
    pub fn new() -> Self {
        Self { tray: None }
    }

    pub fn update_tray(&mut self, config: LinuxTrayConfig) {
        if let Some(ref mut tray) = self.tray {
            tray.update(&config);
        } else {
            let mut tray = LinuxTray::new();
            tray.create_internal(&config);
            self.tray = Some(tray);
        }
    }
}

impl Global for LinuxTrayState {}
