#[derive(Debug, Clone, PartialEq)]
pub enum AriaRole {
    Editor,
    TreeView,
    TabList,
    Tab,
    Menu,
    MenuItem,
    Button,
    TextInput,
    StatusBar,
    Panel,
}

#[derive(Debug, Clone)]
pub struct AccessibleElement {
    pub role: AriaRole,
    pub label: String,
    pub description: Option<String>,
    pub focused: bool,
    pub expanded: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct AccessibilityTree {
    pub elements: Vec<AccessibleElement>,
    pub focus_index: usize,
    pub announcement_queue: Vec<String>,
}

impl Default for AccessibilityTree {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            focus_index: 0,
            announcement_queue: Vec::new(),
        }
    }
}

impl AccessibilityTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_tree() -> Self {
        // Create standard editor elements
        let elements = vec![
            AccessibleElement {
                role: AriaRole::TreeView,
                label: "File Explorer".to_string(),
                description: Some("Project file tree".to_string()),
                focused: false,
                expanded: Some(true),
            },
            AccessibleElement {
                role: AriaRole::TabList,
                label: "Open Editors".to_string(),
                description: None,
                focused: false,
                expanded: None,
            },
            AccessibleElement {
                role: AriaRole::Editor,
                label: "Editor".to_string(),
                description: Some("Code editing area".to_string()),
                focused: true, // Default focus
                expanded: None,
            },
            AccessibleElement {
                role: AriaRole::Panel,
                label: "Terminal".to_string(),
                description: Some("Integrated terminal".to_string()),
                focused: false,
                expanded: Some(false),
            },
            AccessibleElement {
                role: AriaRole::StatusBar,
                label: "Status Bar".to_string(),
                description: None,
                focused: false,
                expanded: None,
            },
        ];

        let mut tree = Self {
            elements,
            focus_index: 2, // Editor is at index 2
            announcement_queue: Vec::new(),
        };
        tree.update_focus();
        tree
    }

    fn update_focus(&mut self) {
        for (i, elem) in self.elements.iter_mut().enumerate() {
            elem.focused = i == self.focus_index;
        }
    }

    pub fn focus_next(&mut self) {
        if self.elements.is_empty() {
            return;
        }
        self.focus_index = (self.focus_index + 1) % self.elements.len();
        self.update_focus();
        self.announce_focused();
    }

    pub fn focus_prev(&mut self) {
        if self.elements.is_empty() {
            return;
        }
        if self.focus_index == 0 {
            self.focus_index = self.elements.len() - 1;
        } else {
            self.focus_index -= 1;
        }
        self.update_focus();
        self.announce_focused();
    }

    pub fn get_focused(&self) -> Option<&AccessibleElement> {
        self.elements.get(self.focus_index)
    }

    pub fn announce(&mut self, message: &str) {
        self.announcement_queue.push(message.to_string());
        // In a real app, this would trigger platform TTS
        // For now we just print to stdout as requested/implied for testing
        // println!("Accessibility Announcement: {}", message);
        // Commented out println to avoid spamming test output, but logic is there
    }

    pub fn announcements(&self) -> &[String] {
        &self.announcement_queue
    }

    fn announce_focused(&mut self) {
        // Need to clone label to avoid borrow checker issues if we used `self.get_focused()` directly with mutable borrow later?
        // Actually `announce` takes `&mut self`. `self.get_focused()` borrows `&self`.
        // So we can't hold `elem` reference while calling `announce`.

        let label = if let Some(elem) = self.elements.get(self.focus_index) {
            Some(elem.label.clone())
        } else {
            None
        };

        if let Some(l) = label {
            let msg = format!("Focused: {}", l);
            self.announce(&msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_navigation() {
        let mut tree = AccessibilityTree::build_tree();

        // Initial focus: Editor (index 2)
        assert_eq!(tree.get_focused().unwrap().label, "Editor");
        assert_eq!(tree.focus_index, 2);

        tree.focus_next(); // -> Panel (index 3)
        assert_eq!(tree.get_focused().unwrap().label, "Terminal");
        assert_eq!(tree.focus_index, 3);

        tree.focus_next(); // -> Status Bar (index 4)
        assert_eq!(tree.get_focused().unwrap().label, "Status Bar");

        tree.focus_next(); // -> File Explorer (index 0)
        assert_eq!(tree.get_focused().unwrap().label, "File Explorer");

        tree.focus_prev(); // -> Status Bar (index 4)
        assert_eq!(tree.get_focused().unwrap().label, "Status Bar");
    }

    #[test]
    fn test_announce_stores_messages() {
        let mut tree = AccessibilityTree::new();
        tree.announce("Test message");

        assert_eq!(tree.announcements().len(), 1);
        assert_eq!(tree.announcements()[0], "Test message");
    }
}
