use forge_confidence::{ConfidenceField, ConfidenceMode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceState {
    Tree(Vec<TreeNode>),
    List(Vec<ListItem>),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub badge: Option<Badge>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub badge: Option<Badge>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub text: Option<String>,
    pub color: String, // Hex code or named color (e.g., "#FF0000", "red")
}

#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    pub project_root: PathBuf,
    pub current_open_file: Option<PathBuf>,
    pub open_files: Vec<PathBuf>,
    pub active_language: Option<String>,
    pub git_branch: Option<String>,
}

pub trait SurfaceIntelligence {
    fn surface_id(&self) -> &str;

    /// Bits consumed from noise budget (lower is better for always-on surfaces)
    fn information_cost(&self) -> f64;

    fn render(&self, confidence: &ConfidenceField, mode: ConfidenceMode) -> SurfaceState;

    fn priority(&self, context: &WorkspaceContext) -> f64;
}
