#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreadcrumbDropdown {
    pub visible: bool,
    pub items: Vec<String>,
    pub selected: usize,
}

impl Default for BreadcrumbDropdown {
    fn default() -> Self {
        Self {
            visible: false,
            items: Vec::new(),
            selected: 0,
        }
    }
}

impl BreadcrumbDropdown {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, items: Vec<String>) {
        self.visible = true;
        self.items = items;
        self.selected = 0;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
        self.selected = 0;
    }

    pub fn select(&mut self, idx: usize) -> Option<&str> {
        if idx < self.items.len() {
            self.selected = idx;
            Some(self.items[idx].as_str())
        } else {
            None
        }
    }

    pub fn move_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.items.len() - 1; // Wrap?
        }
    }

    pub fn move_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
        } else {
            self.selected = 0; // Wrap?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move() {
        let mut bd = BreadcrumbDropdown::new();
        bd.show(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        // Initial 0
        assert_eq!(bd.selected, 0);

        // Down -> 1
        bd.move_down();
        assert_eq!(bd.selected, 1);

        // Down -> 2
        bd.move_down();
        assert_eq!(bd.selected, 2);

        // Down -> 0 (Wrap)
        bd.move_down();
        assert_eq!(bd.selected, 0);

        // Up -> 2 (Wrap)
        bd.move_up();
        assert_eq!(bd.selected, 2);
    }

    #[test]
    fn test_select() {
        let mut bd = BreadcrumbDropdown::new();
        bd.show(vec!["a".to_string()]);
        assert_eq!(bd.select(0), Some("a"));
        assert_eq!(bd.select(1), None);
    }
}
