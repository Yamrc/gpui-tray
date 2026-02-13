//! Windows tray implementation
//!
//! Low-level Windows system tray implementation.
//! Used internally by gpui-tray.

use gpui::{App, BorrowAppContext, Global, MenuItem as GpuiMenuItem, SharedString};
use windows::Win32::Foundation::{FALSE, HWND, TRUE};

use crate::util::encode_wide;
use crate::window::WM_USER_TRAYICON;
use std::sync::atomic::{AtomicU32, Ordering};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Windows tray configuration
pub struct WindowsTrayConfig {
    pub tooltip: Option<SharedString>,
    pub visible: bool,
    pub menu_items: Option<Vec<GpuiMenuItem>>,
}

/// Windows tray implementation
pub struct WindowsTray {
    pub(crate) tray_id: u32,
    pub(crate) hwnd: HWND,
    pub(crate) visible: bool,
}

impl WindowsTray {
    /// Create a new Windows tray
    pub fn new() -> Self {
        Self {
            tray_id: 0,
            hwnd: HWND(std::ptr::null_mut()),
            visible: false,
        }
    }

    /// Set or update the tray for the application
    pub fn set_tray(app: &mut App, config: WindowsTrayConfig) {
        log::info!("Setting up Windows tray...");

        // Get or create the global tray state
        if !app.has_global::<WindowsTrayState>() {
            log::info!("Creating new WindowsTrayState global");
            app.set_global(WindowsTrayState::new());
        }

        // Update the tray using update_global
        app.update_global::<WindowsTrayState, _>(|tray_state, _cx| {
            log::info!("Updating tray via global");
            tray_state.update_tray(config);
        });

        log::info!("Windows tray setup complete");
    }

    pub(crate) fn create_internal(&mut self, config: &WindowsTrayConfig) {
        let tray_id = COUNTER.fetch_add(1, Ordering::Relaxed);
        self.tray_id = tray_id;
        self.visible = config.visible;

        if !config.visible {
            log::info!("Tray not visible, skipping creation");
            return;
        }

        log::info!("Creating Windows tray with ID: {}", tray_id);

        // Create the tray window
        self.hwnd = crate::window::create_tray_window();

        if self.hwnd.is_invalid() {
            log::error!("Failed to create tray window");
            return;
        }

        // Register the tray icon with tooltip
        self.add_tray_icon(config);

        log::info!("Windows tray created successfully");
    }

    fn add_tray_icon(&mut self, config: &WindowsTrayConfig) {
        let tooltip: Option<String> = config.tooltip.as_ref().map(|s| s.to_string());

        log::info!("Adding tray icon with tooltip: {:?}", tooltip);

        let success = self.add_tray_icon_internal(self.hwnd, self.tray_id, tooltip.as_ref());

        if !success {
            log::error!("Failed to add tray icon");
        } else {
            log::info!("Tray icon added successfully");
        }
    }

    fn add_tray_icon_internal(&self, hwnd: HWND, tray_id: u32, tooltip: Option<&String>) -> bool {
        let mut flags = NIF_MESSAGE;
        let mut sz_tip: [u16; 128] = [0; 128];

        if let Some(tip) = tooltip {
            flags |= NIF_TIP;
            let wide_tip = encode_wide(tip);
            for (i, &ch) in wide_tip.iter().take(128).enumerate() {
                sz_tip[i] = ch;
            }
        }

        unsafe {
            let mut nid = NOTIFYICONDATAW {
                uFlags: flags,
                hWnd: hwnd,
                uID: tray_id,
                uCallbackMessage: WM_USER_TRAYICON,
                szTip: sz_tip,
                ..std::mem::zeroed()
            };

            let result = Shell_NotifyIconW(NIM_ADD, &mut nid);
            log::info!("Shell_NotifyIconW(NIM_ADD) result: {:?}", result);
            result == TRUE
        }
    }

    fn modify_tray_icon(&mut self, config: &WindowsTrayConfig) {
        let tooltip: Option<String> = config.tooltip.as_ref().map(|s| s.to_string());

        log::info!("Modifying tray icon with tooltip: {:?}", tooltip);

        let success = self.modify_tray_icon_internal(self.hwnd, self.tray_id, tooltip.as_ref());

        if !success {
            log::error!("Failed to modify tray icon");
        } else {
            log::info!("Tray icon modified successfully");
        }
    }

    fn modify_tray_icon_internal(
        &self,
        hwnd: HWND,
        tray_id: u32,
        tooltip: Option<&String>,
    ) -> bool {
        let mut flags = NIF_MESSAGE;
        let mut sz_tip: [u16; 128] = [0; 128];

        if let Some(tip) = tooltip {
            flags |= NIF_TIP;
            let wide_tip = encode_wide(tip);
            for (i, &ch) in wide_tip.iter().take(128).enumerate() {
                sz_tip[i] = ch;
            }
        }

        unsafe {
            let mut nid = NOTIFYICONDATAW {
                uFlags: flags,
                hWnd: hwnd,
                uID: tray_id,
                uCallbackMessage: WM_USER_TRAYICON,
                szTip: sz_tip,
                ..std::mem::zeroed()
            };

            let result = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
            log::info!("Shell_NotifyIconW(NIM_MODIFY) result: {:?}", result);
            result == TRUE
        }
    }

    fn remove_tray_icon(&self) {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                uFlags: NIF_MESSAGE,
                hWnd: self.hwnd,
                uID: self.tray_id,
                ..std::mem::zeroed()
            };

            if Shell_NotifyIconW(NIM_DELETE, &mut nid) == FALSE {
                log::error!("Error removing system tray icon");
            }
        }
    }

    pub(crate) fn update(&mut self, config: &WindowsTrayConfig) {
        if !config.visible {
            if self.visible {
                log::info!("Hiding tray icon");
                self.remove_tray_icon();
                self.visible = false;
            }
        } else {
            log::info!("Updating existing tray icon");
            self.modify_tray_icon(config);
            self.visible = true;
        }
    }
}

impl Default for WindowsTray {
    fn default() -> Self {
        Self::new()
    }
}

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
            log::info!("Updating existing tray");
            tray.update(&config);
        } else {
            log::info!("Creating new tray");
            let mut tray = WindowsTray::new();
            tray.create_internal(&config);
            self.tray = Some(tray);
        }
    }
}

impl Global for WindowsTrayState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_tray_new() {
        let tray = WindowsTray::new();
        assert_eq!(tray.tray_id, 0);
        assert!(!tray.visible);
    }
}
