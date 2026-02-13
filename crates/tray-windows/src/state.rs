//! Global state management for Windows tray

use gpui::Global;

use crate::tray::{WindowsTray, WindowsTrayConfig};

/// Global state for Windows tray
pub struct WindowsTrayState {
    tray: Option<WindowsTray>,
}

impl WindowsTrayState {
    pub fn new() -> Self {
        Self { tray: None }
    }

    pub fn update_tray(&mut self, config: WindowsTrayConfig) {
        if let Some(ref mut tray) = self.tray {
            log::debug!("Updating existing tray");
            tray.update(&config);
        } else {
            log::debug!("Creating new tray");
            let mut tray = WindowsTray::new();
            tray.create_internal(&config);
            self.tray = Some(tray);
        }
    }
}

impl Global for WindowsTrayState {}
