# Sub-Binary IDE — Integration into Forge Codebase

> **CRITICAL**: Do NOT create a new project. The Forge editor already exists with 7 crates. Add Sub-Binary IDE features as NEW crates within this workspace.

## Existing Workspace (DO NOT MODIFY unless specified)

```
crates/
├── forge-core/       # Rope buffer, transactions, history, syntax (tree-sitter), git, terminal
├── forge-renderer/   # wgpu GPU rendering, text atlas, pipeline, viewport
├── forge-window/     # winit windowing, event loop, input
├── forge-app/        # Main application
├── forge-config/     # TOML configuration
├── forge-theme/      # Color themes
├── forge-input/      # Keyboard/mouse input
```

## Global Rules

1. Rust 2021 edition. No `.unwrap()` in production — `Result<T, E>` + `thiserror`. `.unwrap()` only in `#[cfg(test)]`.
2. Every public function needs ≥1 unit test. `cargo test --workspace` must pass with 0 failures after every task.
3. Run `cargo fmt` + `cargo clippy -- -D warnings` after every file change. Zero warnings.
4. Only add dependencies explicitly listed below.

---

## TASK 1: Add New Crates to Workspace

**Add to root `Cargo.toml`**:

```diff
 [workspace]
 members = [
     "crates/forge-core",
     "crates/forge-renderer",
     "crates/forge-window",
     "crates/forge-app",
     "crates/forge-config",
     "crates/forge-theme",
     "crates/forge-input",
+    "crates/forge-confidence",
+    "crates/forge-propagation",
+    "crates/forge-semantic",
 ]

 [workspace.dependencies]
+# Confidence Engine
+rusqlite = { version = "0.31", features = ["bundled"] }
+petgraph = "0.6"
+uuid = { version = "1", features = ["v4", "serde"] }
+serde_json = "1"
+git2 = "0.19"
```

**Create crate skeletons**:

```
crates/forge-confidence/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── engine.rs        # Weighted sum formula
│   ├── models.rs        # ConfidenceScore, Signal, SignalKind
│   ├── db.rs            # SQLite persistence
│   └── temporal.rs      # Git-based decay

crates/forge-propagation/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── engine.rs        # BFS propagation
│   ├── graph.rs         # Petgraph dependency graph
│   └── models.rs        # FunctionId, EdgeKind, RippleNode

crates/forge-semantic/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── embeddings.rs    # Vector storage + cosine similarity
│   └── intent.rs        # Comment ↔ code alignment
```

**Acceptance**: `cargo check --workspace` succeeds with 0 errors.

---

## TASK 2: Implement Core Data Models

### `forge-confidence/src/models.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub line: usize,
    pub score: f64,
    pub signals: Vec<Signal>,
}

impl ConfidenceScore {
    pub fn new(line: usize, score: f64, signals: Vec<Signal>) -> Self {
        Self { line, score: score.clamp(0.0, 1.0), signals }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub name: SignalKind,
    pub value: f64,
    pub weight: f64,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalKind {
    SyntaxValid,
    TypeCheckPass,
    LintClean,
    TestCovers,
    TestPasses,
    MlBugProbability,
    CodeAgeFactor,
    AuthorExperience,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceUpdate {
    pub file_path: String,
    pub scores: Vec<ConfidenceScore>,
}
```

### `forge-propagation/src/models.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FunctionId {
    pub file: String,
    pub name: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeKind {
    DirectCall,
    TypeDependency,
    ImportSameModule,
    ImportCrossModule,
    TestCoverage,
    InterfaceImpl,
    Inheritance,
}

impl EdgeKind {
    pub fn weight(&self) -> f64 {
        match self {
            Self::DirectCall => 0.50,
            Self::TypeDependency => 0.40,
            Self::ImportSameModule => 0.30,
            Self::ImportCrossModule => 0.20,
            Self::TestCoverage => 0.80,
            Self::InterfaceImpl => 0.60,
            Self::Inheritance => 0.70,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    pub source: FunctionId,
    pub affected: Vec<RippleNode>,
    pub total_files_affected: usize,
    pub total_functions_affected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RippleNode {
    pub function_id: FunctionId,
    pub file_path: String,
    pub delta: f64,
    pub new_confidence: f64,
    pub relationship: EdgeKind,
    pub depth: usize,
}
```

**Tests** (mandatory — add in each file):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_clamps() {
        assert_eq!(ConfidenceScore::new(1, 1.5, vec![]).score, 1.0);
        assert_eq!(ConfidenceScore::new(1, -0.3, vec![]).score, 0.0);
    }

    #[test]
    fn edge_weights() {
        assert_eq!(EdgeKind::DirectCall.weight(), 0.50);
        assert_eq!(EdgeKind::TestCoverage.weight(), 0.80);
    }
}
```

---

## TASK 3: Implement Confidence Engine

### `forge-confidence/src/engine.rs`

**Formula**: `C(line) = Σ(wᵢ × sᵢ)` where `Σ(wᵢ) = 1.0`

Default weights: syntax=0.20, type=0.15, lint=0.10, test_covers=0.20, test_passes=0.10, ml_bug=0.15, age=0.05, author=0.05

**Weight redistribution**: If signal unavailable, redistribute proportionally among available signals.

```rust
pub struct ConfidenceEngine;

impl ConfidenceEngine {
    pub fn compute_line(&self, signals: &[Signal]) -> f64 {
        let available: Vec<_> = signals.iter().filter(|s| s.available).collect();
        let total_weight: f64 = available.iter().map(|s| s.weight).sum();
        if total_weight == 0.0 { return 0.5; }
        available.iter()
            .map(|s| (s.weight / total_weight) * s.value)
            .sum::<f64>()
            .clamp(0.0, 1.0)
    }

    /// CVaR at 10% — average of worst 10% of line scores
    pub fn compute_file(&self, lines: &[ConfidenceScore]) -> f64 {
        if lines.is_empty() { return 0.5; }
        let mut scores: Vec<f64> = lines.iter().map(|l| l.score).collect();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = (scores.len() as f64 * 0.10).ceil().max(1.0) as usize;
        scores[..n].iter().sum::<f64>() / n as f64
    }
}
```

**Tests**:
```rust
#[test]
fn all_perfect_signals() {
    let engine = ConfidenceEngine;
    let signals = vec![
        Signal { name: SignalKind::SyntaxValid, value: 1.0, weight: 0.20, available: true },
        Signal { name: SignalKind::LintClean, value: 1.0, weight: 0.10, available: true },
    ];
    assert!((engine.compute_line(&signals) - 1.0).abs() < 1e-10);
}

#[test]
fn no_signals_returns_half() {
    assert!((ConfidenceEngine.compute_line(&[]) - 0.5).abs() < 1e-10);
}

#[test]
fn cvar_worst_ten_percent() {
    let lines: Vec<ConfidenceScore> = (1..=10)
        .map(|i| ConfidenceScore::new(i, i as f64 * 0.1, vec![]))
        .collect();
    assert!((ConfidenceEngine.compute_file(&lines) - 0.1).abs() < 1e-10);
}
```

---

## TASK 4: Implement Confidence DB (SQLite)

### `forge-confidence/src/db.rs`

**Schema**:
```sql
CREATE TABLE IF NOT EXISTS confidence_scores (
    file_path TEXT NOT NULL,
    line INTEGER NOT NULL,
    score REAL NOT NULL CHECK(score >= 0.0 AND score <= 1.0),
    signals_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (file_path, line)
);
CREATE INDEX IF NOT EXISTS idx_scores_file ON confidence_scores(file_path);
```

```rust
pub struct ConfidenceDb { conn: rusqlite::Connection }

impl ConfidenceDb {
    pub fn open(path: &std::path::Path) -> anyhow::Result<Self>;
    pub fn upsert_scores(&self, file: &str, scores: &[ConfidenceScore]) -> anyhow::Result<()>;
    pub fn get_scores(&self, file: &str) -> anyhow::Result<Vec<ConfidenceScore>>;
    pub fn delete_file(&self, file: &str) -> anyhow::Result<()>;
}
```

- `open()` creates DB + runs schema if needed
- `upsert_scores()` uses `INSERT OR REPLACE` inside a transaction
- Test with `:memory:` SQLite

---

## TASK 5: Implement Graph Store + Propagation Engine

### `forge-propagation/src/graph.rs`

Uses `petgraph::DiGraph`. `add_function()` is idempotent.

```rust
pub struct GraphStore {
    graph: petgraph::graph::DiGraph<FunctionId, EdgeKind>,
    index: std::collections::HashMap<FunctionId, petgraph::graph::NodeIndex>,
}

impl GraphStore {
    pub fn new() -> Self;
    pub fn add_function(&mut self, id: FunctionId) -> petgraph::graph::NodeIndex;
    pub fn add_edge(&mut self, from: &FunctionId, to: &FunctionId, kind: EdgeKind) -> anyhow::Result<()>;
    pub fn get_dependents(&self, id: &FunctionId) -> Vec<(FunctionId, EdgeKind)>;
    pub fn node_count(&self) -> usize;
}
```

### `forge-propagation/src/engine.rs`

**Algorithm**: BFS from source through dependents. `propagated_delta = parent_delta * edge_weight`. Stop when `|delta| < 0.01` or depth > 3.

```rust
pub struct PropagationEngine;

impl PropagationEngine {
    pub fn propagate(
        source: &FunctionId,
        delta: f64,
        graph: &GraphStore,
        scores: &std::collections::HashMap<FunctionId, f64>,
    ) -> PropagationResult;
}
```

**Test**: A→B→C with DirectCall edges. Delta -0.30 at A → B gets -0.15, C gets -0.075.

---

## TASK 6: Implement Temporal Engine

### `forge-confidence/src/temporal.rs`

**Decay**: `C(t) = C_initial × e^(-λt/365)` where `λ = 0.1×authors + 0.05×patch_freq + 0.03×churn`

```rust
pub fn compute_decay(
    author_count: usize, patch_frequency: f64, churn_ratio: f64,
    days_since_commit: f64, initial_confidence: f64,
) -> f64 {
    let lambda = 0.1 * author_count as f64 + 0.05 * patch_frequency + 0.03 * churn_ratio;
    (initial_confidence * (-lambda * days_since_commit / 365.0).exp()).clamp(0.0, 1.0)
}
```

**Test**: `compute_decay(1, 0.5, 0.1, 0.0, 0.95)` returns exactly `0.95` (e^0 = 1).

---

## TASK 7: Wire into forge-app

Update `crates/forge-app/Cargo.toml` to depend on the new crates:
```toml
forge-confidence = { path = "../forge-confidence" }
forge-propagation = { path = "../forge-propagation" }
```

In `forge-app/src/main.rs`, initialize the confidence engine and DB on startup. Hook into the existing buffer change events (from `forge-core`) to trigger confidence recalculation.

---

## Final Verification

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
```

**All must exit 0.**
