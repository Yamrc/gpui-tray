//! GPUI Cross-Platform System Tray Library
//!
//! This crate provides a unified API for system tray functionality
//! across Windows, Linux, and macOS platforms.
//!
//! # Example
//! ```
//! use gpui_tray::{gpui::*, AppTrayExt, Tray};
//!
//! App::new().run(|cx: &mut App| {
//!     let tray = Tray::new()
//!         .tooltip("My App")
//!         .visible(true)
//!         .menu(|_cx| vec![
//!             gpui::MenuItem::action("Show", ShowAction),
//!             gpui::MenuItem::separator(),
//!             gpui::MenuItem::action("Quit", QuitAction),
//!         ]);
//!
//!     cx.set_tray(tray);
//! });
//! ```

pub use gpui;

use gpui::{App, MenuItem as GpuiMenuItem, SharedString};
use std::rc::Rc;

/// Tray icon types
#[derive(Clone, Debug)]
pub enum TrayIcon {
    /// Icon from raw image bytes (PNG format)
    Image { data: Vec<u8> },
    /// Icon from name (Linux uses theme icons)
    Name(String),
}

impl TrayIcon {
    /// Create icon from theme name
    pub fn from_name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }

    /// Create icon from PNG image data
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self::Image { data }
    }
}

/// Mouse button types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Point for position data
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

/// Tray events emitted by user interaction
#[derive(Clone, Debug)]
pub enum TrayEvent {
    /// Tray icon was clicked
    Click {
        button: MouseButton,
        position: Point<i32>,
    },
    /// Tray received scroll input
    Scroll { delta: Point<i32> },
    /// Menu item was selected
    MenuSelect { id: String },
}

/// Internal icon data for platform rendering
#[derive(Clone, Debug)]
pub struct TrayIconData {
    pub data: Rc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

/// System tray configuration
#[derive(Clone)]
pub struct Tray {
    /// Icon to display
    pub icon: Option<TrayIcon>,
    /// Title text (displayed next to icon on macOS)
    pub title: Option<SharedString>,
    /// Tooltip text
    pub tooltip: Option<SharedString>,
    /// Whether the tray icon is visible
    pub visible: bool,
    /// Function to build the context menu
    pub menu_builder: Option<Rc<dyn Fn(&mut App) -> Vec<GpuiMenuItem> + 'static>>,
    /// Internal icon data for platform rendering
    pub icon_data: Option<TrayIconData>,
}

impl Tray {
    /// Create a new tray with default settings
    pub fn new() -> Self {
        Self {
            icon: None,
            title: None,
            tooltip: None,
            visible: true,
            menu_builder: None,
            icon_data: None,
        }
    }

    /// Set the tray icon from GPUI Image
    pub fn icon(mut self, icon: impl Into<gpui::Image>) -> Self {
        // Store placeholder - actual rendering would happen in platform implementation
        self.icon = Some(TrayIcon::Image { data: Vec::new() });
        self
    }

    /// Set the tray title
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the tooltip text
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the context menu builder
    pub fn menu<F>(mut self, builder: F) -> Self
    where
        F: Fn(&mut App) -> Vec<GpuiMenuItem> + 'static,
    {
        self.menu_builder = Some(Rc::new(builder));
        self
    }
}

impl Default for Tray {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Tray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tray")
            .field("icon", &self.icon.is_some())
            .field("title", &self.title)
            .field("tooltip", &self.tooltip)
            .field("visible", &self.visible)
            .finish()
    }
}

/// Extension trait for App to manage system tray
pub trait AppTrayExt {
    /// Set the system tray
    fn set_tray(&mut self, tray: Tray);
}

/// Internal platform trait - implemented by platform-specific crates
pub trait PlatformTray: Send + Sync {
    fn set_tray(&mut self, tray: Tray) -> anyhow::Result<()>;
    fn hide_tray(&mut self) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_builder() {
        let tray = Tray::new()
            .title("Test Title")
            .tooltip("Test Tooltip")
            .visible(true);

        assert_eq!(
            tray.title.as_ref().map(|s| s.to_string()),
            Some("Test Title".to_string())
        );
        assert_eq!(
            tray.tooltip.as_ref().map(|s| s.to_string()),
            Some("Test Tooltip".to_string())
        );
        assert!(tray.visible);
    }

    #[test]
    fn test_tray_default() {
        let tray = Tray::default();
        assert!(tray.icon.is_none());
        assert!(tray.title.is_none());
        assert!(tray.tooltip.is_none());
        assert!(tray.visible);
    }

    #[test]
    fn test_icon_from_name() {
        let icon = TrayIcon::from_name("test-icon");
        match icon {
            TrayIcon::Name(name) => assert_eq!(name, "test-icon"),
            _ => panic!("Expected Name variant"),
        }
    }

    #[test]
    fn test_tray_event_click() {
        let event = TrayEvent::Click {
            button: MouseButton::Left,
            position: Point::new(100, 200),
        };

        match event {
            TrayEvent::Click { button, position } => {
                assert_eq!(button, MouseButton::Left);
                assert_eq!(position.x, 100);
                assert_eq!(position.y, 200);
            }
            _ => panic!("Expected Click variant"),
        }
    }
}
