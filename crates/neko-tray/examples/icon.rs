//! Icon loading example - demonstrates different ways to load icons.

use nekowg::{App, Image, ImageFormat};
use nekowg_platform::application;
use nekowg_tray::{Tray, TrayAppContext};

fn main() {
    application().run(|cx: &mut App| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
        cx.activate(true);

        // Load from embedded bytes
        let icon_from_bytes = Image::from_bytes(
            ImageFormat::Png,
            // include_bytes!("image/app-icon.png").to_vec(),
            // include_bytes!("image/icon.jpg").to_vec(),
            include_bytes!("image/icon.ico").to_vec(),
        );

        // Or load from file at runtime
        // let icon_from_file = Image::from_path("path/to/icon.png").unwrap();

        cx.set_tray(Tray::new().tooltip("Icon Example").icon(icon_from_bytes))
            .unwrap();
    });
}
