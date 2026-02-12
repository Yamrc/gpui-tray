use crate::Tray;
use gpui::App;

/// Extension trait for App to manage system tray
pub trait AppTrayExt {
    /// Set the system tray
    fn set_tray(&mut self, tray: Tray);

    /// Update the system tray
    fn update_tray<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Tray);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_tray_ext_is_trait() {
        // Verify AppTrayExt is a trait (compile-time check)
        fn takes_app_tray_ext<T: AppTrayExt>(_app: &mut T) {}
    }
}
