//! macOS system tray implementation using NSStatusBar

use gpui_tray::{
    gpui::{App, Global, MenuItem as GpuiMenuItem},
    Tray, TrayEvent,
};
use std::sync::Arc;

/// macOS tray implementation using NSStatusBar
pub struct MacosTray {
    visible: bool,
    event_callback: Option<Arc<dyn Fn(TrayEvent) + Send + Sync>>,
}

impl MacosTray {
    /// Create a new macOS tray
    pub fn new() -> Self {
        Self {
            visible: false,
            event_callback: None,
        }
    }

    /// Set the tray for the application
    pub fn set_tray(app: &mut App, tray: Tray) {
        // Get or create the global tray state
        if !app.has_global::<MacosTrayState>() {
            app.set_global(MacosTrayState::new());
        }

        let tray_state = app.global::<MacosTrayState>();
        let menu_items = tray.menu_builder.as_ref().map(|builder| builder(app));

        // Update the tray
        tray_state.update_tray(tray, menu_items);
    }

    fn create_internal(&mut self, tray: &Tray, _menu_items: Option<Vec<GpuiMenuItem>>) {
        self.visible = tray.visible;

        if !tray.visible {
            return;
        }

        // TODO: Implement NSStatusBar
        log::info!("macOS tray created (NSStatusBar implementation pending)");
    }

    fn update(&mut self, tray: &Tray, _menu_items: Option<Vec<GpuiMenuItem>>) {
        self.visible = tray.visible;

        if !tray.visible {
            return;
        }

        log::info!("macOS tray updated");
    }
}

impl Default for MacosTray {
    fn default() -> Self {
        Self::new()
    }
}

/// Global state for macOS tray
pub struct MacosTrayState {
    tray: Option<MacosTray>,
}

impl MacosTrayState {
    pub fn new() -> Self {
        Self { tray: None }
    }

    pub fn update_tray(&mut self, tray_config: Tray, menu_items: Option<Vec<GpuiMenuItem>>) {
        if let Some(ref mut tray) = self.tray {
            tray.update(&tray_config, menu_items);
        } else {
            let mut tray = MacosTray::new();
            tray.create_internal(&tray_config, menu_items);
            self.tray = Some(tray);
        }
    }
}

impl Global for MacosTrayState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_tray_new() {
        let tray = MacosTray::new();
        assert!(!tray.visible);
    }
}
