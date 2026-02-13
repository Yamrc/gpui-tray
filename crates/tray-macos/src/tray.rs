//! macOS tray implementation using NSStatusBar
//!
//! Low-level macOS system tray implementation.
//! Used internally by gpui-tray.

use gpui::{App, BorrowAppContext, Global, MenuItem as GpuiMenuItem, SharedString};

/// macOS tray configuration
#[derive(Clone)]
pub struct MacosTrayConfig {
    pub tooltip: Option<SharedString>,
    pub visible: bool,
    pub menu_items: Option<Vec<GpuiMenuItem>>,
}

/// macOS tray implementation using NSStatusBar
pub struct MacosTray {
    pub(crate) visible: bool,
}

impl MacosTray {
    /// Create a new macOS tray
    pub fn new() -> Self {
        Self { visible: false }
    }

    /// Set the tray for the application
    pub fn set_tray(app: &mut App, config: MacosTrayConfig) {
        // Get or create the global tray state
        if !app.has_global::<crate::state::MacosTrayState>() {
            app.set_global(crate::state::MacosTrayState::new());
        }

        // Update the tray
        app.update_global::<crate::state::MacosTrayState, _>(
            |state: &mut crate::state::MacosTrayState, _cx| {
                state.update_tray(config);
            },
        );
    }

    pub(crate) fn create_internal(&mut self, config: &MacosTrayConfig) {
        self.visible = config.visible;

        if !config.visible {
            return;
        }

        // TODO: Implement NSStatusBar
        log::info!("macOS tray created (NSStatusBar implementation pending)");
    }

    pub(crate) fn update(&mut self, config: &MacosTrayConfig) {
        self.visible = config.visible;

        if !config.visible {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_tray_new() {
        let tray = MacosTray::new();
        assert!(!tray.visible);
    }
}
