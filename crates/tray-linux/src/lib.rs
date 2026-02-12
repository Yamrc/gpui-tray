//! Linux system tray implementation using DBus

use gpui_tray::{
    gpui::{App, Global, MenuItem as GpuiMenuItem},
    Tray, TrayEvent,
};
use std::sync::Arc;

/// Linux tray implementation using DBus StatusNotifierItem
pub struct LinuxTray {
    visible: bool,
    event_callback: Option<Arc<dyn Fn(TrayEvent) + Send + Sync>>,
}

impl LinuxTray {
    /// Create a new Linux tray
    pub fn new() -> Self {
        Self {
            visible: false,
            event_callback: None,
        }
    }

    /// Set the tray for the application
    pub fn set_tray(app: &mut App, tray: Tray) {
        // Get or create the global tray state
        if !app.has_global::<LinuxTrayState>() {
            app.set_global(LinuxTrayState::new());
        }

        let tray_state = app.global::<LinuxTrayState>();
        let menu_items = tray.menu_builder.as_ref().map(|builder| builder(app));

        // Update the tray
        tray_state.update_tray(tray, menu_items);
    }

    fn create_internal(&mut self, tray: &Tray, _menu_items: Option<Vec<GpuiMenuItem>>) {
        self.visible = tray.visible;

        if !tray.visible {
            return;
        }

        // TODO: Implement DBus StatusNotifierItem
        log::info!("Linux tray created (DBus implementation pending)");
    }

    fn update(&mut self, tray: &Tray, _menu_items: Option<Vec<GpuiMenuItem>>) {
        self.visible = tray.visible;

        if !tray.visible {
            return;
        }

        log::info!("Linux tray updated");
    }
}

impl Default for LinuxTray {
    fn default() -> Self {
        Self::new()
    }
}

/// Global state for Linux tray
pub struct LinuxTrayState {
    tray: Option<LinuxTray>,
}

impl LinuxTrayState {
    pub fn new() -> Self {
        Self { tray: None }
    }

    pub fn update_tray(&mut self, tray_config: Tray, menu_items: Option<Vec<GpuiMenuItem>>) {
        if let Some(ref mut tray) = self.tray {
            tray.update(&tray_config, menu_items);
        } else {
            let mut tray = LinuxTray::new();
            tray.create_internal(&tray_config, menu_items);
            self.tray = Some(tray);
        }
    }
}

impl Global for LinuxTrayState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_tray_new() {
        let tray = LinuxTray::new();
        assert!(!tray.visible);
    }
}
