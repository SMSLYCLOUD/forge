# TICKET C3 — Uncertainty Propagation Engine

## Context
- Source: Sessions 9, 17
- Uses Petgraph DiGraph with BFS + damping for propagation across dependency graph

## Requirements
When a file's confidence changes, propagate uncertainty to all dependent files:

### Algorithm
1. Build dependency graph (imports, function calls, type references)
2. When C(file_A) changes by Δ, propagate: `C(file_B) -= Δ × damping^distance`
3. BFS from changed node, damping = 0.7 per hop
4. Max propagation depth = 5 hops
5. Update confidence cache for all affected files

### Data Structures
```rust
use petgraph::graph::DiGraph;

pub struct PropagationEngine {
    pub graph: DiGraph<FileNode, DependencyEdge>,
    pub damping: f64,      // 0.7
    pub max_depth: usize,  // 5
}

pub struct FileNode {
    pub path: PathBuf,
    pub confidence: ConfidenceScore,
}

pub struct DependencyEdge {
    pub kind: DependencyKind,  // Import, Call, Type, Inherit
    pub weight: f64,           // Coupling strength
}
```

## Files to Create
- `forge/crates/forge-propagation/src/lib.rs`
- `forge/crates/forge-propagation/src/engine.rs`
- `forge/crates/forge-propagation/src/graph.rs`
- `forge/crates/forge-propagation/Cargo.toml`
- `forge/crates/forge-propagation/tests/test_propagation.rs`

## Acceptance Criteria
- [ ] Graph built from file dependencies
- [ ] Propagation correct (change in A affects B but not unrelated C)
- [ ] Damping works (distant files less affected)
- [ ] Performance: < 10ms for 1000-file project
- [ ] Tests pass

## Dependencies: C2
## Effort: 3 days → WITH JULES: ~1 session
