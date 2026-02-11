use crate::protocol::{
    ConfidenceField, ConfidenceMode, SurfaceIntelligence, SurfaceState, WorkspaceContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExplorerEntry {
    pub path: String,
    pub confidence: f64,
    pub badge: BadgeColor,
    pub is_relevant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BadgeColor {
    Green,
    Yellow,
    Red,
}

pub struct IntelligentFileExplorer {
    pub files: Vec<String>,
}

impl IntelligentFileExplorer {
    pub fn new(files: Vec<String>) -> Self {
        Self { files }
    }
}

impl SurfaceIntelligence for IntelligentFileExplorer {
    fn surface_id(&self) -> &str {
        "file-explorer"
    }

    fn information_cost(&self) -> f64 {
        15.0 // Moderate cost, main navigation
    }

    fn render(&self, confidence: &ConfidenceField, _mode: ConfidenceMode) -> SurfaceState {
        // 1. Sort files by confidence (worst first)
        // 2. Color code badges

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
                    is_relevant: false, // In real impl, would check relevance to task
                }
            })
            .collect();

        // Sort: Red first (lowest confidence), then Yellow, then Green
        entries.sort_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap());

        // Serialize state for frontend consumption
        let json = serde_json::to_string(&entries).unwrap_or_default();

        SurfaceState {
            content: json,
            priority: 1.0,
            notifications: vec![],
        }
    }

    fn priority(&self, _context: &WorkspaceContext) -> f64 {
        // Always high priority, it's the explorer
        0.9
    }
}
