use crate::{Result, Tray};
use gpui::App;

pub trait PlatformTray {
    fn set_tray(&mut self, cx: &mut App, tray: &Tray) -> Result<()>;
    fn update_tray(&mut self, cx: &mut App, tray: &Tray) -> Result<()>;
    fn remove_tray(&mut self, cx: &mut App) -> Result<()>;
}
