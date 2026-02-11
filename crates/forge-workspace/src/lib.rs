//! Multi-root workspace management.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub roots: Vec<PathBuf>,
}

impl Workspace {
    pub fn from_dir(path: &Path) -> Result<Self> {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        Ok(Self { name, roots: vec![path.to_path_buf()] })
    }

    pub fn add_root(&mut self, path: PathBuf) {
        if !self.roots.contains(&path) { self.roots.push(path); }
    }

    pub fn resolve_path(&self, relative: &str) -> Option<PathBuf> {
        for root in &self.roots {
            let full = root.join(relative);
            if full.exists() { return Some(full); }
        }
        None
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.roots.iter().any(|r| path.starts_with(r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn single_root() {
        let ws = Workspace::from_dir(Path::new(".")).unwrap();
        assert_eq!(ws.roots.len(), 1);
    }
    #[test] fn add_root_dedup() {
        let mut ws = Workspace { name: "test".into(), roots: vec![PathBuf::from("/a")] };
        ws.add_root(PathBuf::from("/a"));
        assert_eq!(ws.roots.len(), 1);
        ws.add_root(PathBuf::from("/b"));
        assert_eq!(ws.roots.len(), 2);
    }
}
