#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: String,
    pub separator: bool,
}

pub struct ContextMenu {
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub items: Vec<MenuItem>,
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self {
            visible: false,
            x: 0.0,
            y: 0.0,
            items: Vec::new(),
        }
    }
}

impl ContextMenu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, x: f32, y: f32, items: Vec<MenuItem>) {
        self.visible = true;
        self.x = x;
        self.y = y;
        self.items = items;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
    }

    pub fn handle_click(&self, idx: usize) -> Option<String> {
        if idx < self.items.len() {
            let item = &self.items[idx];
            if item.separator {
                return None;
            }
            Some(item.action.clone())
        } else {
            None
        }
    }

    pub fn editor_context() -> Vec<MenuItem> {
        vec![
            MenuItem {
                label: "Cut".to_string(),
                shortcut: Some("Ctrl+X".to_string()),
                action: "edit.cut".to_string(),
                separator: false,
            },
            MenuItem {
                label: "Copy".to_string(),
                shortcut: Some("Ctrl+C".to_string()),
                action: "edit.copy".to_string(),
                separator: false,
            },
            MenuItem {
                label: "Paste".to_string(),
                shortcut: Some("Ctrl+V".to_string()),
                action: "edit.paste".to_string(),
                separator: false,
            },
            MenuItem {
                label: "Select All".to_string(),
                shortcut: Some("Ctrl+A".to_string()),
                action: "edit.select_all".to_string(),
                separator: false,
            },
            MenuItem {
                label: "".to_string(),
                shortcut: None,
                action: "".to_string(),
                separator: true,
            },
            MenuItem {
                label: "Go to Definition".to_string(),
                shortcut: Some("F12".to_string()),
                action: "editor.goto_def".to_string(),
                separator: false,
            },
            MenuItem {
                label: "Rename".to_string(),
                shortcut: Some("F2".to_string()),
                action: "editor.rename".to_string(),
                separator: false,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_hide() {
        let mut cm = ContextMenu::new();
        cm.show(10.0, 20.0, ContextMenu::editor_context());

        assert!(cm.visible);
        assert_eq!(cm.items.len(), 7);

        cm.hide();
        assert!(!cm.visible);
        assert!(cm.items.is_empty());
    }

    #[test]
    fn test_handle_click() {
        let mut cm = ContextMenu::new();
        cm.show(0.0, 0.0, ContextMenu::editor_context());

        // Click "Cut" (index 0)
        assert_eq!(cm.handle_click(0), Some("edit.cut".to_string()));

        // Click separator (index 4)
        assert_eq!(cm.handle_click(4), None);
    }
}
