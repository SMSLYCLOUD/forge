use chrono::Utc;
use forge_developer::{BusFactor, DeveloperModel, DeveloperStats, KnowledgeGraph};

#[test]
fn test_developer_integration() {
    // 1. Compute Developer Score
    let now = Utc::now();
    let stats = DeveloperStats {
        commit_count: 50,
        bug_count: 0,
        review_acceptance_rate: 1.0,
        last_edit: now,
        domain_expertise: 0.8,
        flow_score: 0.8,
        fatigue_score: 0.0,
    };
    let score = DeveloperModel::compute_score(&stats, now);
    assert!((score - 0.78).abs() < 1e-6);

    // 2. Build Knowledge Graph
    let mut graph = KnowledgeGraph::new();
    graph.set_confidence("alice", "core", score); // 0.78
    graph.set_confidence("bob", "core", 0.95);
    graph.set_confidence("charlie", "core", 0.60);

    // 3. Compute Bus Factor
    // Threshold 0.8.
    // Alice (0.78) < 0.8.
    // Bob (0.95) >= 0.8.
    // Charlie (0.60) < 0.8.
    // Bus Factor = 1.

    let bf = BusFactor::compute(&graph, "core", 0.8);
    assert_eq!(bf, 1);

    // If Alice improves slightly
    let improved_stats = DeveloperStats {
        commit_count: 70, // 0.7 -> +0.04 base
        ..stats
    };
    // Base = 0.2*0.7 + 0.2 + 0.24 + 0.24 = 0.14 + 0.68 = 0.82
    let new_score = DeveloperModel::compute_score(&improved_stats, now);
    graph.set_confidence("alice", "core", new_score);

    let bf_new = BusFactor::compute(&graph, "core", 0.8);
    assert_eq!(bf_new, 2);
}
