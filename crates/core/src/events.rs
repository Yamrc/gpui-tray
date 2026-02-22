use gpui::*;

#[derive(Clone, PartialEq, Debug, Action)]
#[action(namespace = gpui_tray, no_json)]
pub struct ClickEvent {
    pub button: MouseButton,
    pub position: Point<f32>,
}

#[derive(Clone, PartialEq, Debug, Action)]
#[action(namespace = gpui_tray, no_json)]
pub struct DoubleClickEvent;

#[derive(Clone, PartialEq, Debug, Action)]
#[action(namespace = gpui_tray, no_json)]
pub struct RightClickEvent {
    pub position: Point<f32>,
}

#[derive(Clone, PartialEq, Debug, Action)]
#[action(namespace = gpui_tray, no_json)]
pub struct MouseEnterEvent;

#[derive(Clone, PartialEq, Debug, Action)]
#[action(namespace = gpui_tray, no_json)]
pub struct MouseLeaveEvent;

#[derive(Clone, PartialEq, Debug, Action)]
#[action(namespace = gpui_tray, no_json)]
pub struct ScrollEvent {
    pub delta: Point<f32>,
}
