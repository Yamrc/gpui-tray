use gpui::Action;

/// Menu item kinds
#[derive(Clone)]
pub enum MenuItemKind {
    /// Normal clickable item
    Normal,
    /// Separator line
    Separator,
    /// Checkbox item
    Checkbox { checked: bool },
    /// Radio item
    Radio { selected: bool },
}

/// A menu item
pub struct MenuItem {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Item kind
    pub kind: MenuItemKind,
    /// Whether the item is enabled
    pub enabled: bool,
    /// Optional action to dispatch
    pub action: Option<Box<dyn Action>>,
    /// Submenu items
    pub submenu: Option<Vec<MenuItem>>,
}

impl MenuItem {
    /// Create a new normal menu item
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: MenuItemKind::Normal,
            enabled: true,
            action: None,
            submenu: None,
        }
    }

    /// Create a separator item
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            kind: MenuItemKind::Separator,
            enabled: true,
            action: None,
            submenu: None,
        }
    }

    /// Create a checkbox item
    pub fn checkbox(id: impl Into<String>, label: impl Into<String>, checked: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: MenuItemKind::Checkbox { checked },
            enabled: true,
            action: None,
            submenu: None,
        }
    }

    /// Create a radio item
    pub fn radio(id: impl Into<String>, label: impl Into<String>, selected: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: MenuItemKind::Radio { selected },
            enabled: true,
            action: None,
            submenu: None,
        }
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set action
    pub fn action(mut self, action: Box<dyn Action>) -> Self {
        self.action = Some(action);
        self
    }

    /// Add submenu
    pub fn submenu(mut self, submenu: Vec<MenuItem>) -> Self {
        self.submenu = Some(submenu);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_new() {
        let item = MenuItem::new("id", "Label");
        assert_eq!(item.id, "id");
        assert_eq!(item.label, "Label");
        assert!(matches!(item.kind, MenuItemKind::Normal));
        assert!(item.enabled);
    }

    #[test]
    fn test_menu_item_separator() {
        let item = MenuItem::separator();
        assert!(matches!(item.kind, MenuItemKind::Separator));
    }

    #[test]
    fn test_menu_item_checkbox() {
        let item = MenuItem::checkbox("id", "Label", true);
        assert!(matches!(
            item.kind,
            MenuItemKind::Checkbox { checked: true }
        ));
    }

    #[test]
    fn test_menu_item_radio() {
        let item = MenuItem::radio("id", "Label", true);
        assert!(matches!(item.kind, MenuItemKind::Radio { selected: true }));
    }

    #[test]
    fn test_menu_item_chaining() {
        let item = MenuItem::new("id", "Label").enabled(false);
        assert!(!item.enabled);
    }
}
