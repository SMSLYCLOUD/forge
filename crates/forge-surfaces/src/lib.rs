// Re-export common types
pub use protocol::{SurfaceIntelligence, SurfaceState, ConfidenceMode, WorkspaceContext, ConfidenceField};
pub use file_explorer::IntelligentFileExplorer;

mod protocol;
mod file_explorer;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_explorer_sorting() {
        let mut field = HashMap::new();
        field.insert("bad.rs".to_string(), 0.3);
        field.insert("good.rs".to_string(), 0.9);
        field.insert("meh.rs".to_string(), 0.6);

        let files = vec!["bad.rs".into(), "good.rs".into(), "meh.rs".into()];
        let explorer = IntelligentFileExplorer::new(files);

        let state = explorer.render(&field, ConfidenceMode::Focus);
        let entries: Vec<file_explorer::FileExplorerEntry> = serde_json::from_str(&state.content).unwrap();

        // Should be sorted worst-first
        assert_eq!(entries[0].path, "bad.rs");
        assert_eq!(entries[1].path, "meh.rs");
        assert_eq!(entries[2].path, "good.rs");

        assert_eq!(entries[0].badge, file_explorer::BadgeColor::Red);
        assert_eq!(entries[1].badge, file_explorer::BadgeColor::Yellow);
        assert_eq!(entries[2].badge, file_explorer::BadgeColor::Green);
    }
}
