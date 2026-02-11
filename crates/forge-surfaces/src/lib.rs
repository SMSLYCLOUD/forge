// Re-export common types
pub use file_explorer::IntelligentFileExplorer;
pub use forge_confidence::{ConfidenceField, ConfidenceMode};
pub use protocol::{
    Badge, ListItem, SurfaceIntelligence, SurfaceState, TreeNode, WorkspaceContext,
};

pub mod file_explorer;
mod protocol;
