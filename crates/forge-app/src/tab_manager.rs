use crate::editor::Editor;
use anyhow::Result;
use std::path::PathBuf;

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active: usize,
}

pub struct Tab {
    pub title: String,
    pub path: Option<PathBuf>,
    pub editor: Editor,
    pub is_modified: bool,
}

impl TabManager {
    pub fn new() -> Self { Self { tabs: vec![], active: 0 } }

    pub fn open_file(&mut self, path: &str) -> Result<()> {
        // Don't open duplicate tabs
        if let Some(idx) = self.tabs.iter().position(|t| t.path.as_ref().map(|p| p.to_string_lossy().to_string()).as_deref() == Some(path)) {
            self.active = idx;
            return Ok(());
        }
        let editor = Editor::open_file(path)?;
        let title = std::path::Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "untitled".into());
        self.tabs.push(Tab { title, path: Some(PathBuf::from(path)), editor, is_modified: false });
        self.active = self.tabs.len() - 1;
        Ok(())
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.active >= self.tabs.len() && !self.tabs.is_empty() { self.active = self.tabs.len() - 1; }
        }
    }

    pub fn close_current(&mut self) { let idx = self.active; self.close_tab(idx); }
    pub fn next_tab(&mut self) { if !self.tabs.is_empty() { self.active = (self.active + 1) % self.tabs.len(); } }
    pub fn prev_tab(&mut self) { if !self.tabs.is_empty() { self.active = self.active.checked_sub(1).unwrap_or(self.tabs.len() - 1); } }
    pub fn active_editor(&self) -> Option<&Editor> { self.tabs.get(self.active).map(|t| &t.editor) }
    pub fn active_editor_mut(&mut self) -> Option<&mut Editor> { self.tabs.get_mut(self.active).map(|t| &mut t.editor) }
    pub fn tab_count(&self) -> usize { self.tabs.len() }
}
