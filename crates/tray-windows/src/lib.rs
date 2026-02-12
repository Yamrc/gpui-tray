//! Windows system tray implementation
//!
//! Reference implementation from:
//! https://github.com/tauri-apps/tray-icon

use gpui_tray::{
    Tray, TrayEvent,
    gpui::{App, Global, MenuItem as GpuiMenuItem},
};
use std::{
    ffi::OsStr,
    mem,
    os::windows::ffi::OsStrExt,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicU32, Ordering},
    },
};
use windows::{
    Win32::{
        Foundation::*,
        UI::{Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

const PLATFORM_TRAY_CLASS_NAME: PCWSTR = w!("GPUI::PlatformTray");
const WM_USER_TRAYICON: u32 = 6002;
const WM_USER_UPDATE_TRAYMENU: u32 = 6003;
const WM_USER_UPDATE_TRAYICON: u32 = 6004;
const WM_USER_UPDATE_TRAYTOOLTIP: u32 = 6005;

static WM_TASKBAR_RESTART: LazyLock<u32> =
    LazyLock::new(|| unsafe { RegisterWindowMessageA(s!("TaskbarCreated")) });
static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Windows tray implementation
pub struct WindowsTray {
    tray_id: u32,
    hwnd: HWND,
    hpopupmenu: Option<HMENU>,
    visible: bool,
    event_callback: Option<Arc<dyn Fn(TrayEvent) + Send + Sync>>,
}

impl WindowsTray {
    /// Create a new Windows tray
    pub fn new() -> Self {
        Self {
            tray_id: 0,
            hwnd: HWND(std::ptr::null_mut()),
            hpopupmenu: None,
            visible: false,
            event_callback: None,
        }
    }

    /// Set the tray for the application
    pub fn set_tray(app: &mut App, tray: Tray) {
        // First build the menu before getting mutable access to tray state
        let menu_items = tray.menu_builder.as_ref().map(|builder| builder(app));

        // Get or create the global tray state
        if !app.has_global::<WindowsTrayState>() {
            app.set_global(WindowsTrayState::new());
        }

        // Update the tray
        if let Some(tray_state) = app.try_global::<WindowsTrayState>() {
            // This won't work directly, need to use a different approach
            // For now, just log
            log::info!("Tray would be updated here");
        }
    }

    fn create_internal(&mut self, tray: &Tray, _menu_items: Option<Vec<GpuiMenuItem>>) {
        let tray_id = COUNTER.fetch_add(1, Ordering::Relaxed);
        self.tray_id = tray_id;
        self.visible = tray.visible;

        if !tray.visible {
            return;
        }

        // Create the tray window
        self.hwnd = self.create_tray_window(tray_id);

        // Register the tray icon
        self.register_tray_icon(tray);
    }

    fn create_tray_window(&self, tray_id: u32) -> HWND {
        let tray_data = TrayUserData {
            tray_id,
            hwnd: HWND(std::ptr::null_mut()),
            hpopupmenu: None,
            icon: None,
            tooltip: None,
        };

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
                Some(Box::into_raw(Box::new(tray_data)) as _),
            )
        };

        let hwnd = match hwnd {
            Ok(h) => h,
            Err(e) => {
                log::error!("Failed to create tray window: {:?}", e);
                return HWND(std::ptr::null_mut());
            }
        };

        // Register the initial tray icon
        if !self.register_tray_icon_internal(hwnd, tray_id, None, None) {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
        }

        hwnd
    }

    fn register_tray_icon(&mut self, tray: &Tray) {
        // Convert icon data if available
        let hicon = tray
            .icon_data
            .as_ref()
            .map(|data| create_hicon_from_rgba(&data.data, data.width, data.height))
            .flatten();

        // Convert tooltip
        let tooltip: Option<String> = tray.tooltip.as_ref().map(|s| s.to_string());

        // Register the icon
        self.register_tray_icon_internal(self.hwnd, self.tray_id, hicon, tooltip.as_ref());

        // Store icon handle for later cleanup
        if let Some(icon) = hicon {
            unsafe {
                let _ = SendMessageW(
                    self.hwnd,
                    WM_USER_UPDATE_TRAYICON,
                    Some(WPARAM(Box::into_raw(Box::new(Some(icon))) as usize)),
                    Some(LPARAM(0)),
                );
            }
        }

        // Store tooltip
        if tooltip.is_some() {
            unsafe {
                let _ = SendMessageW(
                    self.hwnd,
                    WM_USER_UPDATE_TRAYTOOLTIP,
                    Some(WPARAM(Box::into_raw(Box::new(tooltip)) as usize)),
                    Some(LPARAM(0)),
                );
            }
        }
    }

    fn register_tray_icon_internal(
        &self,
        hwnd: HWND,
        tray_id: u32,
        hicon: Option<HICON>,
        tooltip: Option<&String>,
    ) -> bool {
        let mut flags = NIF_MESSAGE;
        let mut h_icon = HICON(std::ptr::null_mut());
        let mut sz_tip: [u16; 128] = [0; 128];

        if let Some(icon) = hicon {
            flags |= NIF_ICON;
            h_icon = icon;
        }

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
                hIcon: h_icon,
                szTip: sz_tip,
                ..std::mem::zeroed()
            };

            Shell_NotifyIconW(NIM_ADD, &mut nid) == TRUE
        }
    }

    fn remove_tray_icon(&self) {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                uFlags: NIF_ICON,
                hWnd: self.hwnd,
                uID: self.tray_id,
                ..std::mem::zeroed()
            };

            if Shell_NotifyIconW(NIM_DELETE, &mut nid) == FALSE {
                log::error!("Error removing system tray icon");
            }
        }
    }

    fn update(&mut self, tray: &Tray, _menu_items: Option<Vec<GpuiMenuItem>>) {
        if !tray.visible {
            if self.visible {
                self.remove_tray_icon();
                self.visible = false;
            }
        } else {
            self.register_tray_icon(tray);
            self.visible = true;
        }
    }
}

impl Default for WindowsTray {
    fn default() -> Self {
        Self::new()
    }
}

struct TrayUserData {
    tray_id: u32,
    hwnd: HWND,
    hpopupmenu: Option<HMENU>,
    icon: Option<HICON>,
    tooltip: Option<String>,
}

/// Global state for Windows tray
pub struct WindowsTrayState {
    tray: Option<WindowsTray>,
}

impl WindowsTrayState {
    pub fn new() -> Self {
        Self { tray: None }
    }

    pub fn update_tray(&mut self, tray_config: Tray, menu_items: Option<Vec<GpuiMenuItem>>) {
        if let Some(ref mut tray) = self.tray {
            tray.update(&tray_config, menu_items);
        } else {
            let mut tray = WindowsTray::new();
            tray.create_internal(&tray_config, menu_items);
            self.tray = Some(tray);
        }
    }
}

impl Global for WindowsTrayState {}

fn register_platform_tray_class() {
    static REGISTERED: std::sync::Once = std::sync::Once::new();

    REGISTERED.call_once(|| {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(tray_procedure),
            lpszClassName: PCWSTR(PLATFORM_TRAY_CLASS_NAME.as_ptr()),
            ..Default::default()
        };

        unsafe {
            RegisterClassW(&wc);
        }
    });
}

unsafe extern "system" fn tray_procedure(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let userdata_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };

    let userdata_ptr = match (userdata_ptr, msg) {
        (0, WM_NCCREATE) => unsafe {
            let create_struct = &mut *(lparam.0 as *mut CREATESTRUCTW);
            let userdata = &mut *(create_struct.lpCreateParams as *mut TrayUserData);
            userdata.hwnd = hwnd;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as _);
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        },
        (0, WM_CREATE) => return LRESULT(-1),
        (0, _) => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        _ => userdata_ptr as *mut TrayUserData,
    };

    unsafe {
        let userdata = &mut *(userdata_ptr);

        match msg {
            WM_DESTROY => {
                drop(Box::from_raw(userdata_ptr));
                return LRESULT(0);
            }
            WM_USER_UPDATE_TRAYMENU => {
                let hpopupmenu = Box::from_raw(wparam.0 as *mut Option<HMENU>);
                userdata.hpopupmenu = *hpopupmenu;
            }
            WM_USER_UPDATE_TRAYICON => {
                let icon = Box::from_raw(wparam.0 as *mut Option<HICON>);
                if let Some(old_icon) = userdata.icon {
                    let _ = DestroyIcon(old_icon);
                }
                userdata.icon = *icon;
            }
            WM_USER_UPDATE_TRAYTOOLTIP => {
                let tooltip = Box::from_raw(wparam.0 as *mut Option<String>);
                userdata.tooltip = *tooltip;
            }
            _ if msg == *WM_TASKBAR_RESTART => {
                // Taskbar was restarted
            }
            WM_USER_TRAYICON => {
                let event = lparam.0 as u32;
                match event {
                    WM_RBUTTONDOWN => {
                        let mut cursor_pos = POINT { x: 0, y: 0 };
                        if GetCursorPos(&mut cursor_pos).is_ok() {
                            if let Some(menu) = userdata.hpopupmenu {
                                show_tray_menu(hwnd, menu, cursor_pos.x, cursor_pos.y);
                            }
                        }
                    }
                    WM_LBUTTONUP => {
                        // Left click event
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn show_tray_menu(hwnd: HWND, menu: HMENU, x: i32, y: i32) {
    unsafe {
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(
            menu,
            TPM_BOTTOMALIGN | TPM_LEFTALIGN,
            x,
            y,
            Some(0),
            hwnd,
            None,
        );
    }
}

fn encode_wide<S: AsRef<OsStr>>(string: S) -> Vec<u16> {
    OsStrExt::encode_wide(string.as_ref())
        .chain(std::iter::once(0))
        .collect()
}

#[repr(C)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

fn create_hicon_from_rgba(rgba: &[u8], width: u32, height: u32) -> Option<HICON> {
    const PIXEL_SIZE: usize = mem::size_of::<Pixel>();
    let pixel_count = rgba.len() / PIXEL_SIZE;
    let mut and_mask = Vec::with_capacity(pixel_count);

    let pixels =
        unsafe { std::slice::from_raw_parts_mut(rgba.as_ptr() as *mut Pixel, pixel_count) };

    for pixel in pixels {
        and_mask.push(pixel.a.wrapping_sub(u8::MAX));
    }

    unsafe {
        CreateIcon(
            None,
            width as i32,
            height as i32,
            1,
            (PIXEL_SIZE * 8) as u8,
            and_mask.as_ptr(),
            rgba.as_ptr(),
        )
        .ok()
    }
}

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
