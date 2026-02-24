use gpui::*;
use std::fmt;
use std::sync::Arc;

/// Type alias for menu builder function.
pub type MenuBuilder = Arc<dyn Fn(&mut App) -> Vec<MenuItem> + Send + Sync>;

pub struct Tray {
    pub tooltip: Option<SharedString>,
    pub title: Option<SharedString>,
    pub icon: Option<Image>,
    pub visible: bool,
    pub menu_builder: Option<MenuBuilder>,
}

impl Tray {
    pub fn new() -> Self {
        Self {
            tooltip: None,
            title: None,
            icon: None,
            visible: true,
            menu_builder: None,
        }
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn icon(mut self, icon: Image) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn menu<F>(mut self, builder: F) -> Self
    where
        F: Fn(&mut App) -> Vec<MenuItem> + Send + Sync + 'static,
    {
        self.menu_builder = Some(Arc::new(builder));
        self
    }
}

impl Default for Tray {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Tray {
    fn clone(&self) -> Self {
        Self {
            tooltip: self.tooltip.clone(),
            title: self.title.clone(),
            icon: self.icon.clone(),
            visible: self.visible,
            menu_builder: self.menu_builder.clone(),
        }
    }
}

impl fmt::Debug for Tray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tray")
            .field("tooltip", &self.tooltip)
            .field("title", &self.title)
            .field("visible", &self.visible)
            .field("menu_builder", &self.menu_builder.is_some())
            .finish()
    }
}
