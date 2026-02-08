use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Placeholder for now
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceMode {
    Focus,
    Broad,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceState {
    pub content: String,
    pub priority: f64,
    pub notifications: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceContext {
    pub active_file: Option<String>,
    pub recent_files: Vec<String>,
    pub focus_level: f64,
}

// Map from file path to confidence score
pub type ConfidenceField = HashMap<String, f64>;

pub trait SurfaceIntelligence {
    fn surface_id(&self) -> &str;

    // Bits consumed from noise budget (lower is better for always-on surfaces)
    fn information_cost(&self) -> f64;

    fn render(&self, confidence: &ConfidenceField, mode: ConfidenceMode) -> SurfaceState;

    fn priority(&self, context: &WorkspaceContext) -> f64;
}
