use crate::context_menu::MenuItem;

pub struct Menu {
    pub label: String,
    pub items: Vec<MenuItem>,
}

pub struct TitleBar {
    pub menus: Vec<Menu>,
    pub active_menu: Option<usize>,
}

impl Default for TitleBar {
    fn default() -> Self {
        let mut tb = Self {
            menus: Vec::new(),
            active_menu: None,
        };
        tb.create_defaults();
        tb
    }
}

impl TitleBar {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_defaults(&mut self) {
        self.menus.push(Menu {
            label: "File".to_string(),
            items: vec![
                MenuItem {
                    label: "New File".to_string(),
                    shortcut: Some("Ctrl+N".to_string()),
                    action: "file.new".to_string(),
                    separator: false,
                },
                MenuItem {
                    label: "Open File".to_string(),
                    shortcut: Some("Ctrl+O".to_string()),
                    action: "file.open".to_string(),
                    separator: false,
                },
                MenuItem {
                    label: "Save".to_string(),
                    shortcut: Some("Ctrl+S".to_string()),
                    action: "file.save".to_string(),
                    separator: false,
                },
                MenuItem {
                    label: "Close Editor".to_string(),
                    shortcut: Some("Ctrl+W".to_string()),
                    action: "file.close".to_string(),
                    separator: false,
                },
            ],
        });

        self.menus.push(Menu {
            label: "Edit".to_string(),
            items: vec![
                MenuItem {
                    label: "Undo".to_string(),
                    shortcut: Some("Ctrl+Z".to_string()),
                    action: "edit.undo".to_string(),
                    separator: false,
                },
                MenuItem {
                    label: "Redo".to_string(),
                    shortcut: Some("Ctrl+Y".to_string()),
                    action: "edit.redo".to_string(),
                    separator: false,
                },
                MenuItem {
                    label: "".to_string(),
                    shortcut: None,
                    action: "".to_string(),
                    separator: true,
                },
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
            ],
        });

        self.menus.push(Menu {
            label: "View".to_string(),
            items: vec![
                MenuItem {
                    label: "Command Palette".to_string(),
                    shortcut: Some("Ctrl+Shift+P".to_string()),
                    action: "view.command_palette".to_string(),
                    separator: false,
                },
                MenuItem {
                    label: "Toggle Sidebar".to_string(),
                    shortcut: Some("Ctrl+B".to_string()),
                    action: "view.sidebar".to_string(),
                    separator: false,
                },
                MenuItem {
                    label: "Toggle Terminal".to_string(),
                    shortcut: Some("Ctrl+`".to_string()),
                    action: "view.terminal".to_string(),
                    separator: false,
                },
            ],
        });
    }

    pub fn open_menu(&mut self, idx: usize) {
        if idx < self.menus.len() {
            self.active_menu = Some(idx);
        }
    }

    pub fn close_menu(&mut self) {
        self.active_menu = None;
    }

    pub fn handle_click(&self, item_idx: usize) -> Option<String> {
        if let Some(menu_idx) = self.active_menu {
            if let Some(menu) = self.menus.get(menu_idx) {
                if item_idx < menu.items.len() {
                    let item = &menu.items[item_idx];
                    if item.separator {
                        return None;
                    }
                    return Some(item.action.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let tb = TitleBar::new();
        assert_eq!(tb.menus.len(), 3);
        assert_eq!(tb.menus[0].label, "File");
    }

    #[test]
    fn test_open_close() {
        let mut tb = TitleBar::new();
        tb.open_menu(0);
        assert_eq!(tb.active_menu, Some(0));

        tb.close_menu();
        assert_eq!(tb.active_menu, None);
    }

    #[test]
    fn test_handle_click() {
        let mut tb = TitleBar::new();
        tb.open_menu(0); // File menu

        // Item 0 is New File
        assert_eq!(tb.handle_click(0), Some("file.new".to_string()));
    }
}
