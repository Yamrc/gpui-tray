//! Window creation and message handling for tray

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LRESULT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, CW_USEDEFAULT, WM_LBUTTONUP, WM_RBUTTONDOWN,
    WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_OVERLAPPED,
};

/// Custom window message for tray icon notifications
pub const WM_USER_TRAYICON: u32 = 6002;

/// Window class name for tray window
const PLATFORM_TRAY_CLASS_NAME: PCWSTR = w!("GPUI::PlatformTray");

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
            log::info!("RegisterClassW result: {}", result);
        }
    });
}

/// Create the hidden window for tray message handling
pub fn create_tray_window() -> HWND {
    // Register window class
    register_platform_tray_class();

    // Create the window
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
            log::info!("Tray window created: {:?}", h);
            h
        }
        Err(e) => {
            log::error!("Failed to create tray window: {:?}", e);
            HWND(std::ptr::null_mut())
        }
    }
}

/// Window procedure for tray window
unsafe extern "system" fn tray_procedure(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> LRESULT {
    match msg {
        WM_USER_TRAYICON => {
            let event = lparam.0 as u32;
            match event {
                WM_RBUTTONDOWN => {
                    log::info!("Tray right-click detected");
                }
                WM_LBUTTONUP => {
                    log::info!("Tray left-click detected");
                }
                _ => {}
            }
        }
        _ => {}
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
