#[derive(Debug, Clone, PartialEq)]
pub struct TerminalTab {
    pub id: u64,
    pub title: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct TerminalMultiplexer {
    pub tabs: Vec<TerminalTab>,
    pub active_tab: usize, // Index into tabs
    pub next_id: u64,
}

impl Default for TerminalMultiplexer {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            next_id: 1,
        }
    }
}

impl TerminalMultiplexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_tab(&mut self, title: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let new_tab = TerminalTab {
            id,
            title: title.to_string(),
            active: true, // New tab becomes active
        };

        // Deactivate current active tab
        for tab in &mut self.tabs {
            tab.active = false;
        }

        self.tabs.push(new_tab);
        self.active_tab = self.tabs.len() - 1;

        id
    }

    pub fn close_tab(&mut self, id: u64) -> bool {
        if let Some(index) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(index);

            if self.tabs.is_empty() {
                self.active_tab = 0;
            } else {
                // If we closed the active tab, or a tab before it, we need to adjust active_tab
                if index <= self.active_tab {
                    if self.active_tab > 0 {
                        self.active_tab -= 1;
                    }
                }
                // Make sure the new active tab is marked active
                if self.active_tab < self.tabs.len() {
                    for (i, tab) in self.tabs.iter_mut().enumerate() {
                        tab.active = i == self.active_tab;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    pub fn switch_to(&mut self, id: u64) {
        if let Some(index) = self.tabs.iter().position(|t| t.id == id) {
            self.active_tab = index;
            for (i, tab) in self.tabs.iter_mut().enumerate() {
                tab.active = i == index;
            }
        }
    }

    pub fn active_id(&self) -> Option<u64> {
        if self.active_tab < self.tabs.len() {
            Some(self.tabs[self.active_tab].id)
        } else {
            None
        }
    }

    pub fn rename_tab(&mut self, id: u64, title: &str) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.title = title.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_multiple_tabs() {
        let mut tm = TerminalMultiplexer::new();
        let id1 = tm.create_tab("Term 1");
        assert_eq!(id1, 1);
        assert_eq!(tm.active_id(), Some(1));
        assert!(tm.tabs[0].active);

        let id2 = tm.create_tab("Term 2");
        assert_eq!(id2, 2);
        assert_eq!(tm.active_id(), Some(2));
        assert!(!tm.tabs[0].active);
        assert!(tm.tabs[1].active);
    }

    #[test]
    fn test_switch_between_tabs() {
        let mut tm = TerminalMultiplexer::new();
        let id1 = tm.create_tab("Term 1");
        let id2 = tm.create_tab("Term 2");

        tm.switch_to(id1);
        assert_eq!(tm.active_id(), Some(id1));
        assert!(tm.tabs[0].active);
        assert!(!tm.tabs[1].active);

        tm.switch_to(id2);
        assert_eq!(tm.active_id(), Some(id2));
        assert!(!tm.tabs[0].active);
        assert!(tm.tabs[1].active);
    }

    #[test]
    fn test_close_tab() {
        let mut tm = TerminalMultiplexer::new();
        let id1 = tm.create_tab("Term 1");
        let id2 = tm.create_tab("Term 2");

        assert_eq!(tm.tabs.len(), 2);

        tm.close_tab(id1);
        assert_eq!(tm.tabs.len(), 1);
        assert_eq!(tm.active_id(), Some(id2));

        tm.close_tab(id2);
        assert_eq!(tm.tabs.len(), 0);
        assert_eq!(tm.active_id(), None);
    }

    #[test]
    fn test_rename_tab() {
        let mut tm = TerminalMultiplexer::new();
        let id1 = tm.create_tab("Term 1");
        tm.rename_tab(id1, "New Name");
        assert_eq!(tm.tabs[0].title, "New Name");
    }
}
