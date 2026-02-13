//! Tray event types and input handling

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
    fn test_point_new() {
        let point = Point::new(10, 20);
        assert_eq!(point.x, 10);
        assert_eq!(point.y, 20);
    }
}
