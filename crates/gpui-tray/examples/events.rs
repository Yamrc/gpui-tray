//! Events example - handling tray click events.

use gpui::{App, Application, Image, ImageFormat, actions};
use gpui_tray::{Tray, TrayAppContext};
use gpui_tray_core::{ClickEvent, DoubleClickEvent};
use log::info;

actions!(events_example, [ShowWindow]);

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    Application::new().run(|cx: &mut App| {
        cx.activate(true);

        // Register click event handlers
        cx.on_action(on_tray_click);
        cx.on_action(on_tray_double_click);
        cx.on_action(|_: &ShowWindow, _| println!("Show window!"));

        let icon = Image::from_bytes(
            ImageFormat::Png,
            include_bytes!("image/app-icon.png").to_vec(),
        );

        cx.set_tray(Tray::new().tooltip("Click me!").icon(icon))
            .unwrap();
    });
}

fn on_tray_click(event: &ClickEvent, _cx: &mut App) {
    info!(
        "Tray clicked: button={:?}, position={:?}",
        event.button, event.position
    );

    match event.button {
        gpui::MouseButton::Left => println!("Left click!"),
        gpui::MouseButton::Right => println!("Right click!"),
        gpui::MouseButton::Middle => println!("Middle click!"),
        _ => {}
    }
}

fn on_tray_double_click(_event: &DoubleClickEvent, _cx: &mut App) {
    info!("Tray double-clicked!");
}
