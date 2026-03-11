//! Minimal tray example - just shows a tray icon with tooltip.

use nekowg::{App, Image, ImageFormat};
use nekowg_platform::application;
use nekowg_tray::{Tray, TrayAppContext};

fn main() {
    application().run(|cx: &mut App| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
        cx.activate(true);

        let icon_bytes = include_bytes!("image/app-icon.png");
        let icon = Image::from_bytes(ImageFormat::Png, icon_bytes.to_vec());

        cx.set_tray(Tray::new().tooltip("Hello from GPUI!").icon(icon))
            .unwrap();
    });
}
