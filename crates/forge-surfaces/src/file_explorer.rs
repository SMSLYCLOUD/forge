use crate::{
    Badge, ConfidenceField, ConfidenceMode, SurfaceIntelligence, SurfaceState, TreeNode,
    WorkspaceContext,
};
use std::fs;
use std::path::{Path, PathBuf};

pub struct IntelligentFileExplorer {
    pub root: PathBuf,
}

impl IntelligentFileExplorer {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn scan(&self, path: &Path, confidence: &ConfidenceField) -> Vec<TreeNode> {
        let mut nodes = Vec::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // Skip hidden files/dirs
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }

                let is_dir = entry_path.is_dir();
                let path_str = entry_path.to_string_lossy().to_string();
                let parent_str = entry_path.parent().map(|p| p.to_string_lossy().to_string());

                let mut node = TreeNode {
                    id: path_str.clone(),
                    parent_id: parent_str,
                    label: name.clone(),
                    description: None,
                    icon: Some(if is_dir {
                        "folder".to_string()
                    } else {
                        "file".to_string()
                    }),
                    children: Vec::new(),
                    expanded: false,
                    badge: None,
                    data: None,
                };

                if is_dir {
                    node.children = self.scan(&entry_path, confidence);
                    node.data = Some(serde_json::json!({ "is_dir": true }));
                } else {
                    // It's a file
                    let score = confidence.get(&path_str).copied().unwrap_or(1.0);
                    let color = if score > 0.8 {
                        "#00FF00" // Green
                    } else if score > 0.5 {
                        "#FFFF00" // Yellow
                    } else {
                        "#FF0000" // Red
                    };

                    node.badge = Some(Badge {
                        text: None,
                        color: color.to_string(),
                    });

                    node.data = Some(serde_json::json!({ "score": score, "is_dir": false }));
                }

                nodes.push(node);
            }
        }

        // Sort: Directories first, then files by score (ascending: worst first)
        nodes.sort_by(|a, b| {
            let a_is_dir = a
                .data
                .as_ref()
                .and_then(|d| d.get("is_dir"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let b_is_dir = b
                .data
                .as_ref()
                .and_then(|d| d.get("is_dir"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if a_is_dir != b_is_dir {
                // Dirs first
                return if a_is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }

            if !a_is_dir {
                // Both files: sort by score
                let score_a = a
                    .data
                    .as_ref()
                    .and_then(|d| d.get("score"))
                    .and_then(|s| s.as_f64())
                    .unwrap_or(1.0);
                let score_b = b
                    .data
                    .as_ref()
                    .and_then(|d| d.get("score"))
                    .and_then(|s| s.as_f64())
                    .unwrap_or(1.0);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                // Both dirs: sort by name
                a.label.cmp(&b.label)
            }
        });

        nodes
    }
}

impl SurfaceIntelligence for IntelligentFileExplorer {
    fn surface_id(&self) -> &str {
        "file_explorer"
    }

    fn information_cost(&self) -> f64 {
        0.1
    }

    fn render(&self, confidence: &ConfidenceField, _mode: ConfidenceMode) -> SurfaceState {
        let roots = self.scan(&self.root, confidence);
        SurfaceState::Tree(roots)
    }

    fn priority(&self, context: &WorkspaceContext) -> f64 {
        if context.project_root == self.root {
            1.0
        } else {
            0.0
        }
    }
}
