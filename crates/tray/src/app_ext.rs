//! App extension for system tray support
//!
//! This module implements the platform-agnostic AppTrayExt trait
//! by delegating to platform-specific implementations.

use gpui::{App, MenuItem as GpuiMenuItem};

use crate::types::Tray;

/// Extension trait for App to manage system tray
///
/// This trait provides a unified, platform-agnostic API for setting the system tray.
/// Simply call `cx.set_tray(tray)` from your application.
///
/// # Example
/// ```rust,ignore
/// use gpui::*;
/// use gpui_tray::{Tray, AppTrayExt};
///
/// fn main() {
///     Application::new().run(|cx: &mut App| {
///         let tray = Tray::new()
///             .tooltip("My App")
///             .visible(true)
///             .menu(|_cx| vec![
///                 MenuItem::action("Show", ShowAction),
///                 MenuItem::separator(),
///                 MenuItem::action("Quit", QuitAction),
///             ]);
///         
///         cx.set_tray(tray);
///     });
/// }
/// ```
pub trait AppTrayExt {
    /// Set or update the system tray.
    ///
    /// This method will create the tray if it doesn't exist, or update it if it does.
    /// The platform-specific implementation (Windows/Linux/macOS) is automatically selected
    /// at compile time.
    fn set_tray(&mut self, tray: Tray);
}

impl AppTrayExt for App {
    fn set_tray(&mut self, tray: Tray) {
        // Build menu items
        let menu_items = tray.menu_builder.as_ref().map(|builder| builder(self));

        // Delegate to platform-specific implementation
        set_tray_platform(self, tray, menu_items);
    }
}

/// Platform-specific implementation dispatcher
#[cfg(target_os = "windows")]
fn set_tray_platform(app: &mut App, tray: Tray, menu_items: Option<Vec<GpuiMenuItem>>) {
    use tray_windows::WindowsTrayConfig;

    let config = WindowsTrayConfig {
        tooltip: tray.tooltip,
        visible: tray.visible,
        menu_items,
    };

    tray_windows::WindowsTray::set_tray(app, config);
}

#[cfg(target_os = "linux")]
fn set_tray_platform(app: &mut App, tray: Tray, menu_items: Option<Vec<GpuiMenuItem>>) {
    use tray_linux::LinuxTrayConfig;

    let config = LinuxTrayConfig {
        tooltip: tray.tooltip,
        visible: tray.visible,
        menu_items,
    };

    tray_linux::LinuxTray::set_tray(app, config);
}

#[cfg(target_os = "macos")]
fn set_tray_platform(app: &mut App, tray: Tray, menu_items: Option<Vec<GpuiMenuItem>>) {
    use tray_macos::MacosTrayConfig;

    let config = MacosTrayConfig {
        tooltip: tray.tooltip,
        visible: tray.visible,
        menu_items,
    };

    tray_macos::MacosTray::set_tray(app, config);
}
