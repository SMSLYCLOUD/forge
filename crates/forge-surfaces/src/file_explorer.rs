use crate::protocol::{
    ConfidenceField, ConfidenceMode, SurfaceIntelligence, SurfaceState, WorkspaceContext,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// FileNode — recursive file-system tree for the sidebar explorer
// ---------------------------------------------------------------------------

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
pub enum NodeKind {
    File,
    Directory,
}

impl FileNode {
    pub fn build_tree(root: &Path, max_depth: usize) -> Result<Self> {
        Self::build_recursive(root, 0, max_depth)
    }

    fn build_recursive(path: &Path, depth: usize, max_depth: usize) -> Result<Self> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
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
            Ok(Self {
                name,
                path: path.to_path_buf(),
                kind: NodeKind::Directory,
                children,
                expanded: depth == 0,
                depth,
            })
        } else {
            Ok(Self {
                name,
                path: path.to_path_buf(),
                kind: if path.is_dir() {
                    NodeKind::Directory
                } else {
                    NodeKind::File
                },
                children: vec![],
                expanded: false,
                depth,
            })
        }
    }

    pub fn toggle(&mut self, target: &Path) -> bool {
        if self.path == target {
            self.expanded = !self.expanded;
            return true;
        }
        for child in &mut self.children {
            if child.toggle(target) {
                return true;
            }
        }
        false
    }

    pub fn flatten_visible(&self) -> Vec<&FileNode> {
        let mut result = vec![self];
        if self.expanded {
            for child in &self.children {
                result.extend(child.flatten_visible());
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// IntelligentFileExplorer — confidence-aware file explorer surface
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BadgeColor {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExplorerEntry {
    pub path: String,
    pub confidence: f64,
    pub badge: BadgeColor,
    pub is_relevant: bool,
}

pub struct IntelligentFileExplorer {
    files: Vec<String>,
}

impl IntelligentFileExplorer {
    pub fn new(files: Vec<String>) -> Self {
        Self { files }
    }
}

impl SurfaceIntelligence for IntelligentFileExplorer {
    fn surface_id(&self) -> &str {
        "file_explorer"
    }

    fn information_cost(&self) -> f64 {
        0.3 // Low cost — always visible in sidebar
    }

    fn render(&self, confidence: &ConfidenceField, _mode: ConfidenceMode) -> SurfaceState {
        let mut entries: Vec<FileExplorerEntry> = self
            .files
            .iter()
            .map(|f| {
                let score = *confidence.get(f).unwrap_or(&0.5);
                let badge = if score > 0.8 {
                    BadgeColor::Green
                } else if score > 0.5 {
                    BadgeColor::Yellow
                } else {
                    BadgeColor::Red
                };

                FileExplorerEntry {
                    path: f.clone(),
                    confidence: score,
                    badge,
                    is_relevant: false,
                }
            })
            .collect();

        // Sort worst-first so attention goes to risky files
        entries.sort_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap());

        SurfaceState {
            content: serde_json::to_string(&entries).unwrap_or_default(),
            priority: entries.first().map(|e| 1.0 - e.confidence).unwrap_or(0.0),
            notifications: entries
                .iter()
                .filter(|e| e.confidence < 0.4)
                .map(|e| format!("⚠ {} confidence: {:.0}%", e.path, e.confidence * 100.0))
                .collect(),
        }
    }

    fn priority(&self, _context: &WorkspaceContext) -> f64 {
        0.5 // Medium priority — always relevant
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_build_tree() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(file_path).unwrap();
        let sub_dir = dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        let tree = FileNode::build_tree(dir.path(), 5).unwrap();
        assert_eq!(tree.kind, NodeKind::Directory);
        assert_eq!(tree.children.len(), 2);
    }

    #[test]
    fn test_toggle() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();

        let mut tree = FileNode::build_tree(dir.path(), 5).unwrap();
        // Root starts expanded
        assert!(tree.expanded);

        // Find child path
        let child_path = tree.children[0].path.clone();

        // Toggle child
        tree.toggle(&child_path);
        assert!(tree.children[0].expanded);

        // Toggle back
        tree.toggle(&child_path);
        assert!(!tree.children[0].expanded);
    }
}
