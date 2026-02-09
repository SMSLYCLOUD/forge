# Agent 09 — forge-workspace + Project Detection

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: forge-workspace Crate

### Create `crates/forge-workspace/Cargo.toml`
```toml
[package]
name = "forge-workspace"
version.workspace = true
edition.workspace = true
[dependencies]
anyhow = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }
```

### `crates/forge-workspace/src/lib.rs`
```rust
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
```

## Task B: Enhanced Project Detection

### `crates/forge-core/src/project.rs` — enhance existing file
Add project type detection. Keep existing code, add these functions:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectKind { Rust, Node, Python, Go, Generic }

pub fn detect_project_kind(root: &Path) -> ProjectKind {
    if root.join("Cargo.toml").exists() { ProjectKind::Rust }
    else if root.join("package.json").exists() { ProjectKind::Node }
    else if root.join("pyproject.toml").exists() || root.join("setup.py").exists() { ProjectKind::Python }
    else if root.join("go.mod").exists() { ProjectKind::Go }
    else { ProjectKind::Generic }
}
```

Add `"crates/forge-workspace"` to workspace members + dependencies.

**Acceptance**: `cargo test -p forge-workspace -p forge-core` passes.
