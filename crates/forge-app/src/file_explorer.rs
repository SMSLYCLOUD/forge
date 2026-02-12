use crate::file_tree_ui::DisplayNode;
use std::path::{Path, PathBuf};

pub struct FileExplorer {
    pub root: Option<PathBuf>,
    pub nodes: Vec<DisplayNode>,
    pub paths: Vec<PathBuf>,
    pub selected: Option<usize>,
}

impl FileExplorer {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: Vec::new(),
            paths: Vec::new(),
            selected: None,
        }
    }

    pub fn scan_directory(&mut self, root: &Path) -> anyhow::Result<()> {
        self.root = Some(root.to_path_buf());
        self.nodes.clear();
        self.paths.clear();
        self.scan_recursive(root, 0)?;
        Ok(())
    }

    fn scan_recursive(&mut self, dir: &Path, depth: usize) -> anyhow::Result<()> {
        if depth > 3 {
            return Ok(());
        } // Limit depth
        let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
        entries.sort_by(|a, b| {
            let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
            b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
        });
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let path = entry.path();
            self.nodes.push(DisplayNode {
                label: name,
                depth,
                is_dir,
                expanded: depth == 0 && is_dir,
            });
            self.paths.push(path.clone());
            if is_dir && depth == 0 {
                let _ = self.scan_recursive(&path, depth + 1);
            }
        }
        Ok(())
    }

    pub fn get_path(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(|p| p.as_path())
    }

    pub fn toggle_expand(&mut self, _index: usize) {
        if let Some(node) = self.nodes.get_mut(_index) {
            if node.is_dir {
                node.expanded = !node.expanded;
            }
        }
    }
}

impl Default for FileExplorer {
    fn default() -> Self {
        Self::new()
    }
}
