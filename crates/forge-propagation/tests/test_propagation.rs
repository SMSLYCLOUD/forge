
use forge_propagation::{GraphStore, PropagationEngine, FileNode, DependencyKind};

#[test]
fn test_propagation_chain() {
    let mut graph = GraphStore::new();

    // A -> B -> C
    // A imports B
    // B calls C
    // C is the source of change

    let a_idx = graph.add_file("A.rs".to_string(), 0.9);
    let b_idx = graph.add_file("B.rs".to_string(), 0.8);
    let c_idx = graph.add_file("C.rs".to_string(), 0.7);

    // B -> C (Call)
    graph.add_dependency("B.rs", "C.rs", DependencyKind::Call).unwrap();
    // A -> B (Import)
    graph.add_dependency("A.rs", "B.rs", DependencyKind::Import).unwrap();

    let engine = PropagationEngine::new();

    // Change C by +0.1
    let source_node = FileNode { path: "C.rs".to_string(), confidence: 0.7 };
    let result = engine.propagate(&source_node, 0.1, &graph);

    // Expect B and A to be affected
    assert_eq!(result.affected.len(), 2);

    // B is affected by C (Call weight 0.5) * damping (0.7)
    // 0.1 * 0.5 * 0.7 = 0.035
    // But logic applies it as *reduction* due to uncertainty
    let b_effect = result.affected.iter().find(|n| n.path == "B.rs").unwrap();
    assert!((b_effect.delta + 0.035).abs() < 0.001); // -0.035

    // A is affected by B (Import weight 0.3) * damping (0.7)
    // 0.035 * 0.3 * 0.7 = 0.00735
    let a_effect = result.affected.iter().find(|n| n.path == "A.rs").unwrap();
    assert!((a_effect.delta + 0.00735).abs() < 0.001); // -0.00735
}

#[test]
fn test_damping_limit() {
    let engine = PropagationEngine {
        damping: 0.1, // Very high damping
        max_depth: 5,
    };
    // ... test logic
}
