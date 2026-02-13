//! Tray event types and input handling

pub use gpui::Point;

/// Mouse button types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_tray_event_scroll() {
        let event = TrayEvent::Scroll {
            delta: Point::new(5, -3),
        };

        match event {
            TrayEvent::Scroll { delta } => {
                assert_eq!(delta.x, 5);
                assert_eq!(delta.y, -3);
            }
            _ => panic!("Expected Scroll variant"),
        }
    }

    #[test]
    fn test_tray_event_menu_select() {
        let event = TrayEvent::MenuSelect {
            id: String::from("item-1"),
        };

        match event {
            TrayEvent::MenuSelect { id } => {
                assert_eq!(id, "item-1");
            }
            _ => panic!("Expected MenuSelect variant"),
        }
    }
}
