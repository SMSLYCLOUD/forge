use forge_anticipation::{FileState, LayoutNode, SplitDirection, WorkspaceSnapshot};
use std::collections::HashMap;

#[test]
fn test_snapshot_roundtrip() {
    let mut files = HashMap::new();
    files.insert(
        "src/main.rs".to_string(),
        FileState {
            path: "src/main.rs".to_string(),
            cursor_line: 10,
            cursor_col: 5,
            scroll_top: 0,
            scroll_left: 0,
        },
    );

    let layout = LayoutNode::Split {
        direction: SplitDirection::Horizontal,
        children: vec![
            LayoutNode::Leaf {
                file_path: Some("src/main.rs".to_string()),
            },
            LayoutNode::Leaf { file_path: None },
        ],
        sizes: vec![0.7, 0.3],
    };

    let snapshot = WorkspaceSnapshot {
        branch: "feature/foo".to_string(),
        files,
        layout,
        focused_file: Some("src/main.rs".to_string()),
    };

    let path = "test_snapshot.json";
    snapshot.save_to_file(path).unwrap();

    let loaded = WorkspaceSnapshot::load_from_file(path).unwrap();

    assert_eq!(loaded.branch, "feature/foo");
    assert_eq!(loaded.files.len(), 1);
    assert_eq!(loaded.focused_file, Some("src/main.rs".to_string()));

    // Check layout structure
    if let LayoutNode::Split {
        direction,
        children,
        ..
    } = loaded.layout
    {
        assert!(matches!(direction, SplitDirection::Horizontal));
        assert_eq!(children.len(), 2);
    } else {
        panic!("Layout structure mismatch");
    }

    std::fs::remove_file(path).unwrap();
}
