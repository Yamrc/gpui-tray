//! Window creation and message handling for tray

use gpui::MenuItem as GpuiMenuItem;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CW_USEDEFAULT, CreatePopupMenu, CreateWindowExW, DefWindowProcW, GetCursorPos,
    HMENU, MF_SEPARATOR, MF_STRING, RegisterClassW, SetForegroundWindow, TPM_BOTTOMALIGN,
    TPM_LEFTALIGN, TrackPopupMenu, WM_LBUTTONUP, WM_MBUTTONUP, WM_RBUTTONUP, WNDCLASSW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_OVERLAPPED,
};
use windows::core::{PCWSTR, w};

/// Custom window message for tray icon notifications
pub const WM_USER_TRAYICON: u32 = 6002;

/// Window class name for tray window
const PLATFORM_TRAY_CLASS_NAME: PCWSTR = w!("GPUI::Tray");

/// Tray user data stored in window
pub struct TrayUserData {
    pub hmenu: Option<HMENU>,
}

/// Register the window class for tray window
fn register_platform_tray_class() {
    static REGISTERED: std::sync::Once = std::sync::Once::new();

    REGISTERED.call_once(|| {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(tray_procedure),
            lpszClassName: PCWSTR(PLATFORM_TRAY_CLASS_NAME.as_ptr()),
            ..Default::default()
        };

        unsafe {
            let result = RegisterClassW(&wc);
            log::debug!("RegisterClassW result: {}", result);
        }
    });
}

/// Create the hidden window for tray message handling
pub fn create_tray_window() -> HWND {
    register_platform_tray_class();

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
            PLATFORM_TRAY_CLASS_NAME,
            None,
            WS_OVERLAPPED,
            CW_USEDEFAULT,
            0,
            CW_USEDEFAULT,
            0,
            None,
            None,
            None,
            None,
        )
    };

    match hwnd {
        Ok(h) => {
            log::debug!("window created: {:?}", h);
            h
        }
        Err(e) => {
            log::error!("Failed to create tray window: {:?}", e);
            HWND(std::ptr::null_mut())
        }
    }
}

/// Build Windows HMENU from GPUI MenuItems
pub fn build_menu(items: &[GpuiMenuItem]) -> Option<HMENU> {
    unsafe {
        let hmenu = CreatePopupMenu().ok()?;

        for (index, item) in items.iter().enumerate() {
            let id = index + 1; // Menu item ID (1-based, 0 is reserved)

            match item {
                GpuiMenuItem::Separator => {
                    let _ = AppendMenuW(hmenu, MF_SEPARATOR, id, windows::core::PCWSTR::null());
                }
                GpuiMenuItem::Action { name, .. } => {
                    let wide_name: Vec<u16> = OsStr::new(name.as_ref())
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();
                    let result = AppendMenuW(
                        hmenu,
                        MF_STRING,
                        id,
                        windows::core::PCWSTR(wide_name.as_ptr()),
                    );
                    if result.is_err() {
                        log::error!("Failed to append menu item: {}", name);
                    }
                }
                GpuiMenuItem::Submenu(submenu) => {
                    // TODO: Implement submenu support
                    log::warn!("Submenu not yet implemented: {}", submenu.name);
                }
                _ => {
                    log::warn!("Unsupported menu item type");
                }
            }
        }

        Some(hmenu)
    }
}

/// Show tray context menu at cursor position
pub fn show_tray_menu(hwnd: HWND, hmenu: HMENU) {
    unsafe {
        let mut cursor_pos = windows::Win32::Foundation::POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor_pos).is_ok() {
            let _ = SetForegroundWindow(hwnd);
            let _ = TrackPopupMenu(
                hmenu,
                TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                cursor_pos.x,
                cursor_pos.y,
                Some(0),
                hwnd,
                None,
            );
        }
    }
}

/// Window procedure for tray window
/// TODO: Handle event
unsafe extern "system" fn tray_procedure(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_USER_TRAYICON => {
            let event = lparam.0 as u32;

            match event {
                WM_LBUTTONUP => {
                    log::info!("WM_LBUTTONUP detected");
                }
                WM_RBUTTONUP => {
                    log::info!("WM_RBUTTONUP detected");

                    unsafe {
                        let user_data_ptr =
                            windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                                hwnd,
                                windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                            );
                        if user_data_ptr != 0 {
                            let user_data = &*(user_data_ptr as *const TrayUserData);
                            if let Some(hmenu) = user_data.hmenu {
                                show_tray_menu(hwnd, hmenu);
                            }
                        }
                    }
                }
                WM_MBUTTONUP => {
                    log::info!("WM_MBUTTONUP detected");
                }
                _ => {}
            }
        }
        _ => {}
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
