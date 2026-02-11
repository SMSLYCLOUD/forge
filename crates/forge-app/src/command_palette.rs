#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub category: Option<String>,
}

pub struct CommandPalette {
    pub visible: bool,
    pub query: String,
    pub commands: Vec<Command>,
    pub filtered: Vec<usize>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        let mut cp = Self {
            visible: false,
            query: String::new(),
            commands: Vec::new(),
            filtered: Vec::new(),
        };
        cp.register_defaults();
        cp.filter(); // Initial filter (all)
        cp
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        Self::default()
    }

    fn register_defaults(&mut self) {
        let defaults = vec![
            ("file.new", "File: New File", Some("Ctrl+N"), "File"),
            ("file.open", "File: Open File", Some("Ctrl+O"), "File"),
            ("file.save", "File: Save", Some("Ctrl+S"), "File"),
            (
                "file.save_as",
                "File: Save As",
                Some("Ctrl+Shift+S"),
                "File",
            ),
            ("file.close", "File: Close Editor", Some("Ctrl+W"), "File"),
            ("file.quit", "File: Quit", Some("Ctrl+Q"), "File"),
            ("edit.undo", "Edit: Undo", Some("Ctrl+Z"), "Edit"),
            ("edit.redo", "Edit: Redo", Some("Ctrl+Y"), "Edit"),
            ("edit.cut", "Edit: Cut", Some("Ctrl+X"), "Edit"),
            ("edit.copy", "Edit: Copy", Some("Ctrl+C"), "Edit"),
            ("edit.paste", "Edit: Paste", Some("Ctrl+V"), "Edit"),
            ("edit.find", "Edit: Find", Some("Ctrl+F"), "Edit"),
            ("edit.replace", "Edit: Replace", Some("Ctrl+H"), "Edit"),
            (
                "view.command_palette",
                "View: Command Palette",
                Some("Ctrl+Shift+P"),
                "View",
            ),
            (
                "view.terminal",
                "View: Toggle Terminal",
                Some("Ctrl+`"),
                "View",
            ),
            (
                "view.sidebar",
                "View: Toggle Sidebar",
                Some("Ctrl+B"),
                "View",
            ),
            ("view.minimap", "View: Toggle Minimap", None, "View"),
            ("help.about", "Help: About", None, "Help"),
            // Add more to reach 30+
            (
                "editor.format",
                "Editor: Format Document",
                Some("Shift+Alt+F"),
                "Editor",
            ),
            (
                "editor.fold",
                "Editor: Fold",
                Some("Ctrl+Shift+["),
                "Editor",
            ),
            (
                "editor.unfold",
                "Editor: Unfold",
                Some("Ctrl+Shift+]"),
                "Editor",
            ),
            (
                "editor.comment",
                "Editor: Toggle Comment",
                Some("Ctrl+/"),
                "Editor",
            ),
            ("git.commit", "Git: Commit", None, "Git"),
            ("git.push", "Git: Push", None, "Git"),
            ("git.pull", "Git: Pull", None, "Git"),
            ("debug.start", "Debug: Start Debugging", Some("F5"), "Debug"),
            (
                "debug.stop",
                "Debug: Stop Debugging",
                Some("Shift+F5"),
                "Debug",
            ),
            ("debug.step_over", "Debug: Step Over", Some("F10"), "Debug"),
            ("debug.step_into", "Debug: Step Into", Some("F11"), "Debug"),
            (
                "debug.step_out",
                "Debug: Step Out",
                Some("Shift+F11"),
                "Debug",
            ),
        ];

        for (id, label, shortcut, category) in defaults {
            self.commands.push(Command {
                id: id.to_string(),
                label: label.to_string(),
                shortcut: shortcut.map(|s| s.to_string()),
                category: Some(category.to_string()),
            });
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.filter();
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.filtered.clear();
    }

    pub fn type_char(&mut self, c: char) {
        self.query.push(c);
        self.filter();
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.filter();
    }

    pub fn select(&self, idx: usize) -> Option<&Command> {
        if idx < self.filtered.len() {
            let command_idx = self.filtered[idx];
            self.commands.get(command_idx)
        } else {
            None
        }
    }

    fn filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.commands.len()).collect();
            return;
        }

        let mut scored: Vec<(usize, i32)> = Vec::new();

        for (i, cmd) in self.commands.iter().enumerate() {
            if let Some(score) = Self::fuzzy_score(&self.query, &cmd.label) {
                scored.push((i, score));
            }
        }

        // Sort by score descending
        scored.sort_by(|a, b| b.1.cmp(&a.1));

        self.filtered = scored.into_iter().map(|(i, _)| i).collect();
    }

    fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
        let query_lower = query.to_lowercase();
        let target_lower = target.to_lowercase();

        let mut score = 0;
        let mut last_match_idx: Option<usize> = None; // Index in Vec<char>

        let target_chars: Vec<char> = target_lower.chars().collect();
        let mut target_cursor = 0;

        for qc in query_lower.chars() {
            let mut found = false;
            // Find char in remaining target
            while target_cursor < target_chars.len() {
                let tc = target_chars[target_cursor];
                if tc == qc {
                    found = true;
                    // Score
                    score += target_chars.len() as i32 - target_cursor as i32;

                    if let Some(last) = last_match_idx {
                        if target_cursor == last + 1 {
                            score += 10;
                        } else {
                            score -= 1;
                        }
                    }

                    last_match_idx = Some(target_cursor);
                    target_cursor += 1;
                    break;
                }
                target_cursor += 1;
            }
            if !found {
                return None;
            }
        }
        Some(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_fuzzy() {
        let mut cp = CommandPalette::new();
        // Defaults include "File: Open File", "File: Save"

        cp.type_char('f');
        cp.type_char('i');
        cp.type_char('l');
        // "fil" matches "File: ..."

        assert!(!cp.filtered.is_empty());

        let first = cp.select(0).unwrap();
        assert!(first.label.to_lowercase().contains("file"));
    }

    #[test]
    fn test_defaults() {
        let cp = CommandPalette::new();
        assert!(cp.commands.len() >= 30);
    }

    #[test]
    fn test_select() {
        let mut cp = CommandPalette::new();
        cp.open();
        // Assuming filtered contains all
        let cmd = cp.select(0);
        assert!(cmd.is_some());
    }
}
