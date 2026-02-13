//! Global state management for macOS tray

use gpui::Global;

use crate::tray::{MacosTray, MacosTrayConfig};

/// Global state for macOS tray
pub struct MacosTrayState {
    tray: Option<MacosTray>,
}

impl MacosTrayState {
    pub fn new() -> Self {
        Self { tray: None }
    }

    pub fn update_tray(&mut self, config: MacosTrayConfig) {
        if let Some(ref mut tray) = self.tray {
            tray.update(&config);
        } else {
            let mut tray = MacosTray::new();
            tray.create_internal(&config);
            self.tray = Some(tray);
        }
    }
}

impl Global for MacosTrayState {}
