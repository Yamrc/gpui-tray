//! Linux tray implementation using DBus StatusNotifierItem
//!
//! Low-level Linux system tray implementation.
//! Used internally by gpui-tray.

use gpui::{App, BorrowAppContext, Global, MenuItem as GpuiMenuItem, SharedString};

/// Linux tray configuration
#[derive(Clone)]
pub struct LinuxTrayConfig {
    pub tooltip: Option<SharedString>,
    pub visible: bool,
    pub menu_items: Option<Vec<GpuiMenuItem>>,
}

/// Linux tray implementation using DBus StatusNotifierItem
pub struct LinuxTray {
    pub(crate) visible: bool,
}

impl LinuxTray {
    /// Create a new Linux tray
    pub fn new() -> Self {
        Self { visible: false }
    }

    /// Set the tray for the application
    pub fn set_tray(app: &mut App, config: LinuxTrayConfig) {
        // Get or create the global tray state
        if !app.has_global::<crate::state::LinuxTrayState>() {
            app.set_global(crate::state::LinuxTrayState::new());
        }

        // Update the tray
        app.update_global::<crate::state::LinuxTrayState, _>(
            |state: &mut crate::state::LinuxTrayState, _cx| {
                state.update_tray(config);
            },
        );
    }

    pub(crate) fn create_internal(&mut self, config: &LinuxTrayConfig) {
        self.visible = config.visible;

        if !config.visible {
            return;
        }

        // TODO: Implement DBus StatusNotifierItem
        log::info!("Linux tray created (DBus implementation pending)");
    }

    pub(crate) fn update(&mut self, config: &LinuxTrayConfig) {
        self.visible = config.visible;

        if !config.visible {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_tray_new() {
        let tray = LinuxTray::new();
        assert!(!tray.visible);
    }
}
