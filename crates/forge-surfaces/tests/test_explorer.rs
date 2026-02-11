use forge_surfaces::{
    ConfidenceField, ConfidenceMode, IntelligentFileExplorer, SurfaceIntelligence, SurfaceState,
    TreeNode, WorkspaceContext,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_explorer_sorting() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();

    // Create files
    fs::write(root.join("bad.rs"), "bad").unwrap();
    fs::write(root.join("good.rs"), "good").unwrap();
    fs::write(root.join("meh.rs"), "meh").unwrap();
    fs::create_dir(root.join("subdir")).unwrap();
    fs::write(root.join("subdir/sub.rs"), "sub").unwrap();

    let mut confidence = ConfidenceField::new();
    confidence.insert(root.join("bad.rs").to_string_lossy().to_string(), 0.3);
    confidence.insert(root.join("good.rs").to_string_lossy().to_string(), 0.9);
    confidence.insert(root.join("meh.rs").to_string_lossy().to_string(), 0.6);

    let explorer = IntelligentFileExplorer::new(root.clone());

    // Test priority
    let context = WorkspaceContext {
        project_root: root.clone(),
        current_open_file: None,
        open_files: vec![],
        active_language: None,
        git_branch: None,
    };
    assert_eq!(explorer.priority(&context), 1.0);

    // Test render
    let state = explorer.render(&confidence, ConfidenceMode::Realtime);

    if let SurfaceState::Tree(nodes) = state {
        // Expected order: subdir (dir), bad.rs (0.3), meh.rs (0.6), good.rs (0.9)
        // Check dir first
        // Note: sorting order logic puts dirs first.
        // Assuming subdir is the only dir.
        let dirs: Vec<&TreeNode> = nodes.iter().filter(|n| !n.children.is_empty()).collect();
        let files: Vec<&TreeNode> = nodes.iter().filter(|n| n.children.is_empty()).collect();

        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].label, "subdir");

        assert_eq!(files.len(), 3);
        assert_eq!(files[0].label, "bad.rs");
        assert_eq!(files[1].label, "meh.rs");
        assert_eq!(files[2].label, "good.rs");

        // Check badges
        assert_eq!(files[0].badge.as_ref().unwrap().color, "#FF0000"); // Red
        assert_eq!(files[1].badge.as_ref().unwrap().color, "#FFFF00"); // Yellow
        assert_eq!(files[2].badge.as_ref().unwrap().color, "#00FF00"); // Green
    } else {
        panic!("Expected SurfaceState::Tree");
    }
}
