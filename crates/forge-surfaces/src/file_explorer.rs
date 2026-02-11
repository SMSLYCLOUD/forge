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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;

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
