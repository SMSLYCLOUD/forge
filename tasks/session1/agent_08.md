# Agent 08 — Tab Manager + File I/O

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Multi-Tab File Manager

### `crates/forge-app/src/tab_manager.rs`
```rust
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
```

## Task B: Robust File I/O

### `crates/forge-core/src/file_io.rs`
```rust
use anyhow::Result;
use std::path::Path;

pub struct FileIO;

impl FileIO {
    /// Atomic save: write to temp, then rename.
    pub fn save_atomic(path: &Path, content: &str) -> Result<()> {
        let tmp = path.with_extension("forge-tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Detect if file is binary (contains null bytes in first 8KB).
    pub fn is_binary(path: &Path) -> Result<bool> {
        let data = std::fs::read(path)?;
        let check_len = data.len().min(8192);
        Ok(data[..check_len].contains(&0))
    }

    /// Detect line ending style from content.
    pub fn detect_line_ending(content: &str) -> &'static str {
        if content.contains("\r\n") { "\r\n" } else { "\n" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn atomic_save() {
        let dir = std::env::temp_dir().join("forge_io_test");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("test.txt");
        FileIO::save_atomic(&path, "hello").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
        std::fs::remove_dir_all(&dir).ok();
    }
    #[test] fn detect_lf() { assert_eq!(FileIO::detect_line_ending("a\nb\nc"), "\n"); }
    #[test] fn detect_crlf() { assert_eq!(FileIO::detect_line_ending("a\r\nb\r\nc"), "\r\n"); }
}
```

Add `pub mod file_io;` to `forge-core/src/lib.rs`. Add `mod tab_manager;` to `main.rs`.

**Acceptance**: `cargo test -p forge-core -p forge-app` passes.
