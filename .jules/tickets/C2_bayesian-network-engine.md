# TICKET C2 — Bayesian Network Inference Engine (Rust)

## Context
- Source: Sessions 10, 17
- Discovery: `discoveries/2026-02-08_sub-binary-ide-beyond-limits.md`
- Mega-synthesis: `discoveries/2026-02-08_mega-synthesis-sub-binary-ide.md`
- Core equation: `SHIP ⟺ CVaR_α( P(correct(L, t+τ, K) | BayesNet(E₁..Eₙ)) ) ≥ τ_ship`

## Requirements
Implement Bayesian Network inference for the confidence engine:

### Core Algorithm
- Variable elimination for DAGs (exact inference)
- Loopy Belief Propagation for cyclic graphs (approximate)
- Junction tree fallback when LBP doesn't converge (50-iteration limit)

### Data Structures
```rust
pub struct BayesNet {
    pub nodes: Vec<BayesNode>,
    pub edges: Vec<(usize, usize)>,  // parent → child
}

pub struct BayesNode {
    pub id: usize,
    pub name: String,
    pub cpt: ConditionalProbabilityTable,
    pub observed: Option<f64>,
}

pub struct ConditionalProbabilityTable {
    pub parents: Vec<usize>,
    pub probabilities: Vec<f64>,  // Flattened table
}
```

### Functions
- `variable_elimination(net: &BayesNet, query: usize) -> f64`
- `belief_propagation(net: &BayesNet, max_iter: usize) -> Vec<f64>`
- `junction_tree_infer(net: &BayesNet, query: usize) -> f64`
- `infer(net: &BayesNet, query: usize) -> f64` — auto-selects algorithm

### Performance Target
- O(64) per line inference (6 criteria × ~10 evidence nodes)
- < 1ms per file (1000 lines)

## Files to Create
- `forge/crates/forge-bayesnet/src/lib.rs`
- `forge/crates/forge-bayesnet/src/network.rs`
- `forge/crates/forge-bayesnet/src/variable_elimination.rs`
- `forge/crates/forge-bayesnet/src/belief_propagation.rs`
- `forge/crates/forge-bayesnet/src/junction_tree.rs`
- `forge/crates/forge-bayesnet/Cargo.toml`
- `forge/crates/forge-bayesnet/tests/test_inference.rs`
- `forge/crates/forge-bayesnet/benches/bench_inference.rs`

## Acceptance Criteria
- [ ] Variable elimination correct on known test networks (Asia, Alarm)
- [ ] LBP converges on acyclic graphs
- [ ] Junction tree handles cyclic graphs
- [ ] Performance: < 1ms per 1000-line file
- [ ] `cargo test` passes (20+ test cases)
- [ ] `cargo bench` shows performance numbers

## Effort: 5 days → WITH JULES: ~2 sessions
