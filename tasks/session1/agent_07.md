# Agent 07 — Real File Tree + File Tree UI

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Real File Explorer

Replace placeholder in `crates/forge-surfaces/src/file_explorer.rs` with real recursive directory walking.

```rust
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub kind: NodeKind,
    pub children: Vec<FileNode>,
    pub expanded: bool,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind { File, Directory }

impl FileNode {
    pub fn build_tree(root: &Path, max_depth: usize) -> Result<Self> {
        Self::build_recursive(root, 0, max_depth)
    }

    fn build_recursive(path: &Path, depth: usize, max_depth: usize) -> Result<Self> {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if path.is_dir() && depth < max_depth {
            let mut children: Vec<FileNode> = std::fs::read_dir(path)?
                .filter_map(|e| e.ok())
                .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                .filter(|e| e.file_name() != "target" && e.file_name() != "node_modules")
                .filter_map(|e| Self::build_recursive(&e.path(), depth + 1, max_depth).ok())
                .collect();
            children.sort_by(|a, b| match (&a.kind, &b.kind) {
                (NodeKind::Directory, NodeKind::File) => std::cmp::Ordering::Less,
                (NodeKind::File, NodeKind::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });
            Ok(Self { name, path: path.to_path_buf(), kind: NodeKind::Directory, children, expanded: depth == 0, depth })
        } else {
            Ok(Self { name, path: path.to_path_buf(), kind: if path.is_dir() { NodeKind::Directory } else { NodeKind::File }, children: vec![], expanded: false, depth })
        }
    }

    pub fn toggle(&mut self, target: &Path) -> bool {
        if self.path == target { self.expanded = !self.expanded; return true; }
        for child in &mut self.children { if child.toggle(target) { return true; } }
        false
    }

    pub fn flatten_visible(&self) -> Vec<&FileNode> {
        let mut result = vec![self];
        if self.expanded {
            for child in &self.children { result.extend(child.flatten_visible()); }
        }
        result
    }
}
```

## Task B: File Tree UI Renderer

### `crates/forge-app/src/file_tree_ui.rs`
```rust
use crate::rect_renderer::Rect;

pub struct FileTreeUi {
    pub scroll_offset: usize,
    pub selected_index: Option<usize>,
    pub hovered_index: Option<usize>,
}

impl FileTreeUi {
    pub fn new() -> Self { Self { scroll_offset: 0, selected_index: None, hovered_index: None } }

    pub fn render_rects(&self, nodes: &[DisplayNode], zone: &crate::ui::Zone) -> Vec<Rect> {
        let mut rects = Vec::new();
        let line_h = 22.0;
        for (i, node) in nodes.iter().enumerate() {
            let y = zone.y + (i as f32 * line_h);
            if y > zone.y + zone.height { break; }
            if Some(i) == self.hovered_index {
                rects.push(Rect { x: zone.x, y, w: zone.width, h: line_h, color: [255, 255, 255, 15] });
            }
            if Some(i) == self.selected_index {
                rects.push(Rect { x: zone.x, y, w: zone.width, h: line_h, color: [0, 120, 215, 60] });
            }
        }
        rects
    }
}

pub struct DisplayNode {
    pub label: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}
```

Add `mod file_tree_ui;` to main.rs. Tests for FileNode::build_tree with temp dirs.

**Acceptance**: `cargo test -p forge-surfaces -p forge-app` passes.
