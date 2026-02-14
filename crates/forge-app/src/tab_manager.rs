use crate::editor::Editor;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pane {
    Primary,
    Secondary,
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active: usize,
    pub active_secondary: Option<usize>,
    pub focused_pane: Pane,
}

pub struct Tab {
    pub title: String,
    pub path: Option<PathBuf>,
    pub editor: Editor,
    pub is_modified: bool,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: vec![],
            active: 0,
            active_secondary: None,
            focused_pane: Pane::Primary,
        }
    }

    /// Open a scratch (untitled) tab so keyboard input works immediately.
    pub fn open_scratch(&mut self) {
        let mut editor = Editor::new();
        let welcome_text = "Forge IDE\n\nStart\n  New File          (Ctrl+N)\n  Open File...      (Ctrl+O)\n  Open Folder...    (Ctrl+K Ctrl+O)\n\nHelp\n  Show All Commands  (Ctrl+Shift+P)\n  Terminal            (Ctrl+`)";

        let change = forge_core::Change::insert(forge_core::Position::new(0), welcome_text.to_string());
        let tx = forge_core::Transaction::new(forge_core::ChangeSet::with_change(change), None);
        editor.buffer.apply(tx);

        self.tabs.push(Tab {
            title: "Welcome".to_string(),
            path: None,
            editor,
            is_modified: false,
        });
        self.active = 0;
    }

    pub fn open_file(&mut self, path: &str) -> Result<()> {
        // Don't open duplicate tabs
        if let Some(idx) = self.tabs.iter().position(|t| {
            t.path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .as_deref()
                == Some(path)
        }) {
            self.active = idx;
            return Ok(());
        }
        let editor = Editor::open_file(path)?;
        let title = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".into());
        self.tabs.push(Tab {
            title,
            path: Some(PathBuf::from(path)),
            editor,
            is_modified: false,
        });
        self.active = self.tabs.len() - 1;
        Ok(())
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.active >= self.tabs.len() && !self.tabs.is_empty() {
                self.active = self.tabs.len() - 1;
            }
        }
    }

    pub fn close_current(&mut self) {
        let idx = self.active;
        self.close_tab(idx);
    }
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active + 1) % self.tabs.len();
        }
    }
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = self.active.checked_sub(1).unwrap_or(self.tabs.len() - 1);
        }
    }
    pub fn active_editor(&self) -> Option<&Editor> {
        let idx = match self.focused_pane {
            Pane::Primary => self.active,
            Pane::Secondary => self.active_secondary.unwrap_or(self.active),
        };
        self.tabs.get(idx).map(|t| &t.editor)
    }
    pub fn active_editor_mut(&mut self) -> Option<&mut Editor> {
        let idx = match self.focused_pane {
            Pane::Primary => self.active,
            Pane::Secondary => self.active_secondary.unwrap_or(self.active),
        };
        self.tabs.get_mut(idx).map(|t| &mut t.editor)
    }
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn split_current(&mut self) {
        let idx = match self.focused_pane {
            Pane::Primary => self.active,
            Pane::Secondary => self.active_secondary.unwrap_or(self.active),
        };
        if idx < self.tabs.len() {
            let tab = &self.tabs[idx];
            let new_editor = tab.editor.clone_view();
            self.tabs.push(Tab {
                title: tab.title.clone(),
                path: tab.path.clone(),
                editor: new_editor,
                is_modified: tab.is_modified,
            });
            self.active_secondary = Some(self.tabs.len() - 1);
            self.focused_pane = Pane::Secondary;
        }
    }

    pub fn sync_buffers(&mut self) {
        let active_idx = match self.focused_pane {
            Pane::Primary => self.active,
            Pane::Secondary => self.active_secondary.unwrap_or(self.active),
        };

        if active_idx >= self.tabs.len() {
            return;
        }

        let path_opt = self.tabs[active_idx].path.clone();
        if let Some(path) = path_opt {
            let indices: Vec<usize> = self
                .tabs
                .iter()
                .enumerate()
                .filter(|(i, t)| *i != active_idx && t.path.as_ref() == Some(&path))
                .map(|(i, _)| i)
                .collect();

            if indices.is_empty() {
                return;
            }

            let source_buffer = self.tabs[active_idx].editor.buffer.clone();

            for i in indices {
                self.tabs[i].editor.buffer.sync_content_from(&source_buffer);
                self.tabs[i].editor.rehighlight();
                self.tabs[i].is_modified = self.tabs[active_idx].is_modified;
            }
        }
    }

    pub fn mark_active_modified(&mut self) {
        let idx = match self.focused_pane {
            Pane::Primary => self.active,
            Pane::Secondary => self.active_secondary.unwrap_or(self.active),
        };
        if let Some(tab) = self.tabs.get_mut(idx) {
            tab.is_modified = true;
        }
        self.sync_buffers();
    }
}
