use gpui::{MenuItem as GpuiMenuItem, MouseButton};
use log::{debug, error};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

use crate::util::encode_wide;

pub const WM_USER_TRAYICON: u32 = 6002;
pub const WM_USER_SET_MENU: u32 = WM_USER + 1;
pub const WM_USER_DESTROY_MENU: u32 = WM_USER + 2;

const PLATFORM_TRAY_CLASS_NAME: PCWSTR = w!("GPUI::Tray");

static CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);

pub trait TrayEventDispatcher: Send + Sync + 'static {
    fn dispatch_click(&self, button: MouseButton, position: gpui::Point<f32>);
    fn dispatch_double_click(&self);
    fn dispatch_menu_action(&self, action: Box<dyn gpui::Action>);
}

thread_local! {
    static DISPATCHER: Cell<Option<&'static dyn TrayEventDispatcher>> = Cell::new(None);
    static MENU_ACTIONS: Cell<Option<&'static HashMap<u32, Box<dyn gpui::Action>>>> = Cell::new(None);
}

pub fn set_dispatcher(dispatcher: Option<&'static dyn TrayEventDispatcher>) {
    DISPATCHER.set(dispatcher);
}

pub fn set_menu_actions(actions: Option<&'static HashMap<u32, Box<dyn gpui::Action>>>) {
    MENU_ACTIONS.set(actions);
}

fn dispatch_click(button: MouseButton, position: gpui::Point<f32>) {
    DISPATCHER.with(|cell| {
        if let Some(dispatcher) = cell.get() {
            dispatcher.dispatch_click(button, position);
        }
    });
}

fn dispatch_double_click() {
    DISPATCHER.with(|cell| {
        if let Some(dispatcher) = cell.get() {
            dispatcher.dispatch_double_click();
        }
    });
}

fn dispatch_menu_action(action_id: u32) {
    MENU_ACTIONS.with(|cell| {
        if let Some(actions) = cell.get() {
            if let Some(action) = actions.get(&action_id) {
                DISPATCHER.with(|dispatcher_cell| {
                    if let Some(dispatcher) = dispatcher_cell.get() {
                        dispatcher.dispatch_menu_action(action.boxed_clone());
                    }
                });
            }
        }
    });
}

struct TrayUserData {
    hmenu: Option<HMENU>,
}

unsafe extern "system" fn tray_procedure(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if msg == WM_NCCREATE {
            let user_data = Box::new(TrayUserData { hmenu: None });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(user_data) as isize);
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        let user_data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);

        if user_data_ptr == 0 {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        let user_data = &mut *(user_data_ptr as *mut TrayUserData);

        match msg {
            WM_DESTROY => {
                debug!("Window WM_DESTROY received, cleaning up");
                if let Some(hmenu) = user_data.hmenu.take() {
                    if DestroyMenu(hmenu).is_err() {
                        error!("Failed to destroy menu during window cleanup");
                    }
                }
                let _ = Box::from_raw(user_data_ptr as *mut TrayUserData);
                LRESULT(0)
            }
            WM_USER_SET_MENU => {
                let menu_ptr = lparam.0;
                debug!("Received WM_USER_SET_MENU, menu_ptr: {:?}", menu_ptr);
                user_data.hmenu = if menu_ptr == 0 {
                    None
                } else {
                    Some(HMENU(menu_ptr as *mut _))
                };
                LRESULT(0)
            }
            WM_USER_DESTROY_MENU => {
                debug!("Received WM_USER_DESTROY_MENU");
                if let Some(hmenu) = user_data.hmenu.take() {
                    if DestroyMenu(hmenu).is_err() {
                        error!("Failed to destroy menu");
                    } else {
                        debug!("Menu destroyed successfully");
                    }
                }
                LRESULT(0)
            }
            WM_USER_TRAYICON => {
                let event = lparam.0 as u32;
                let mut pos = POINT { x: 0, y: 0 };
                let has_pos = GetCursorPos(&mut pos).is_ok();
                let position = gpui::Point::new(pos.x as f32, pos.y as f32);
                match event {
                    WM_LBUTTONDOWN => {
                        debug!(
                            "Received WM_LBUTTONDOWN with position: ({}, {})",
                            pos.x, pos.y
                        );
                        if has_pos {
                            dispatch_click(MouseButton::Left, position);
                        }
                    }
                    WM_LBUTTONDBLCLK => {
                        debug!(
                            "Received WM_LBUTTONDBLCLK with position: ({}, {})",
                            pos.x, pos.y
                        );
                        dispatch_double_click();
                    }
                    WM_MBUTTONUP => {
                        debug!(
                            "Received WM_MBUTTONUP with position: ({}, {})",
                            pos.x, pos.y
                        );
                        if has_pos {
                            dispatch_click(MouseButton::Middle, position);
                        }
                    }
                    WM_RBUTTONUP => {
                        debug!(
                            "Received WM_RBUTTONUP with position: ({}, {})",
                            pos.x, pos.y
                        );
                        if has_pos {
                            dispatch_click(MouseButton::Right, position);
                        }
                        if let Some(hmenu) = user_data.hmenu {
                            show_tray_menu(hwnd, hmenu);
                        }
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let command_id = wparam.0 as u32;
                debug!("Received WM_COMMAND with ID: {}", command_id);
                dispatch_menu_action(command_id);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn register_platform_tray_class() -> Result<(), &'static str> {
    if CLASS_REGISTERED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(tray_procedure),
            lpszClassName: PLATFORM_TRAY_CLASS_NAME,
            ..Default::default()
        };

        unsafe {
            let result = RegisterClassW(&wc);
            if result == 0 {
                error!("Failed to register window class");
                CLASS_REGISTERED.store(false, Ordering::SeqCst);
                return Err("Failed to register window class");
            }
            debug!("Window class registered successfully, atom: {}", result);
        }
    }
    Ok(())
}

pub fn create_tray_window() -> Result<HWND, &'static str> {
    register_platform_tray_class()?;

    let hwnd_result = unsafe {
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

    match hwnd_result {
        Ok(h) => {
            debug!("Created tray window successfully: {:?}", h);
            Ok(h)
        }
        Err(e) => {
            error!("Failed to create tray window: {:?}", e);
            Err("Failed to create tray window")
        }
    }
}

pub unsafe fn build_menu(
    items: &[GpuiMenuItem],
) -> Option<(HMENU, Vec<(u32, Box<dyn gpui::Action>)>)> {
    unsafe {
        let hmenu = CreatePopupMenu().ok()?;
        let mut actions = Vec::new();
        build_menu_items(hmenu, items, 0, &mut actions);
        Some((hmenu, actions))
    }
}

unsafe fn build_menu_items(
    hmenu: HMENU,
    items: &[GpuiMenuItem],
    start_id: u32,
    actions: &mut Vec<(u32, Box<dyn gpui::Action>)>,
) -> u32 {
    unsafe {
        let mut current_id = start_id;

        for item in items {
            match item {
                GpuiMenuItem::Separator => {
                    let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                }
                GpuiMenuItem::Action { name, action, .. } => {
                    current_id += 1;
                    let wide_name = encode_wide(name.as_ref());
                    let result = AppendMenuW(
                        hmenu,
                        MF_STRING,
                        current_id as usize,
                        PCWSTR(wide_name.as_ptr()),
                    );
                    if result.is_err() {
                        error!("Failed to append menu item: {}", name);
                    } else {
                        actions.push((current_id, action.boxed_clone()));
                    }
                }
                GpuiMenuItem::Submenu(submenu) => {
                    if let Ok(submenu_handle) = CreatePopupMenu() {
                        let next_id =
                            build_menu_items(submenu_handle, &submenu.items, current_id, actions);
                        current_id = next_id;

                        let wide_name = encode_wide(submenu.name.as_ref());
                        let result = AppendMenuW(
                            hmenu,
                            MF_POPUP,
                            submenu_handle.0 as usize,
                            PCWSTR(wide_name.as_ptr()),
                        );

                        if result.is_err() {
                            error!("Failed to append submenu: {}", submenu.name);
                            let _ = DestroyMenu(submenu_handle);
                        }
                    }
                }
                _ => {}
            }
        }

        current_id
    }
}

fn show_tray_menu(hwnd: HWND, hmenu: HMENU) {
    unsafe {
        let mut cursor_pos = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor_pos).is_ok() {
            let _ = SetForegroundWindow(hwnd);
            let result = TrackPopupMenu(
                hmenu,
                TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                cursor_pos.x,
                cursor_pos.y,
                Some(0),
                hwnd,
                None,
            );
            debug!("TrackPopupMenu result: {:?}", result);
            let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        }
    }
}

pub fn set_window_menu(hwnd: HWND, hmenu: Option<HMENU>) {
    if hwnd.is_invalid() {
        error!("Attempted to set menu on invalid window");
        return;
    }

    unsafe {
        let menu_ptr = hmenu.map(|h| h.0 as isize).unwrap_or(0);
        debug!("Sending WM_USER_SET_MENU with menu_ptr: {:?}", menu_ptr);
        SendMessageW(
            hwnd,
            WM_USER_SET_MENU,
            Some(WPARAM(0)),
            Some(LPARAM(menu_ptr)),
        );
    }
}

pub fn destroy_window_menu(hwnd: HWND) {
    if hwnd.is_invalid() {
        return;
    }

    unsafe {
        debug!("Sending WM_USER_DESTROY_MENU");
        SendMessageW(hwnd, WM_USER_DESTROY_MENU, Some(WPARAM(0)), Some(LPARAM(0)));
    }
}

pub fn destroy_window(hwnd: HWND) -> bool {
    if hwnd.is_invalid() {
        debug!("Window already invalid, skipping destroy");
        return true;
    }

    unsafe {
        match DestroyWindow(hwnd) {
            Ok(_) => {
                debug!("Window destroyed successfully");
                true
            }
            Err(e) => {
                error!("Failed to destroy window: {:?}", e);
                false
            }
        }
    }
}
