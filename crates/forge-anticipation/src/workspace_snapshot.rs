use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceSnapshot {
    pub branch: String,
    pub files: HashMap<String, FileState>,
    pub layout: LayoutNode,
    pub focused_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileState {
    pub path: String,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_top: usize,
    pub scroll_left: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum LayoutNode {
    Leaf { file_path: Option<String> },
    Split {
        direction: SplitDirection,
        children: Vec<LayoutNode>,
        sizes: Vec<f32>
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

impl WorkspaceSnapshot {
    pub fn new(branch: String, layout: LayoutNode) -> Self {
        Self {
            branch,
            files: HashMap::new(),
            layout,
            focused_file: None,
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let snapshot = serde_json::from_str(&content)?;
        Ok(snapshot)
    }
}
