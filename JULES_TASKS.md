# Sub-Binary IDE — Exhaustive Implementation Task List for Jules

> **Goal**: Build the Sub-Binary IDE from scratch. Every task is atomic, has exact acceptance criteria, exact file paths, exact data structures, and exact test commands. If you follow this document literally, the result is bug-free.

---

## ⚙️ Global Rules (Apply to EVERY task)

1. **Language**: Rust 2021 edition for all backend code via Tauri v2. TypeScript strict mode for all frontend code. Python 3.11+ for ML server.
2. **Error Handling**: NEVER use `.unwrap()` in production code. Use `Result<T, E>` with `thiserror` for custom errors. `?` operator for propagation. `.unwrap()` is ONLY allowed in `#[cfg(test)]` blocks.
3. **Testing floor**: Every public function MUST have ≥1 unit test. Every engine MUST have ≥1 integration test. Run `cargo test --workspace` after every task — it MUST pass with 0 failures.
4. **Naming**: snake_case for Rust, camelCase for TypeScript, snake_case for Python. No abbreviations except standard ones (e.g., `db`, `lsp`, `ast`).
5. **Dependencies**: Only add a dependency if explicitly listed below. Do NOT add unlisted crates/npm packages.
6. **Formatting**: Run `cargo fmt` and `cargo clippy -- -D warnings` after every Rust file change. Zero warnings allowed.

---

## MILESTONE 1 — "It Works" (Foundation)

### TASK 1.1: Initialize Tauri v2 Project

**What**: Create the project skeleton using Tauri CLI.

**Steps**:
```bash
cargo install create-tauri-app
cargo create-tauri-app subbinary-ide --template vanilla-ts
cd subbinary-ide
```

**Acceptance**:
- [ ] `cargo tauri dev` launches a window with "Hello World"
- [ ] Project structure matches:
```
subbinary-ide/
├── Cargo.toml          # workspace root
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/main.rs
│   ├── tauri.conf.json
├── src/                # frontend
│   ├── index.html
│   ├── main.ts
│   └── styles.css
```

**Exact `Cargo.toml` workspace dependencies** (add these):
```toml
[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.31", features = ["bundled"] }
petgraph = "0.6"
tree-sitter = "0.24"
tree-sitter-typescript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-rust = "0.23"
tree-sitter-javascript = "0.23"
git2 = "0.19"
log = "0.4"
env_logger = "0.11"
uuid = { version = "1", features = ["v4", "serde"] }
```

---

### TASK 1.2: Create Rust Module Skeleton

**What**: Create all Rust source files with empty module declarations. No logic yet — just the file tree and `mod` statements.

**Create these files** (all empty except `mod.rs` files):

```
src-tauri/src/
├── main.rs
├── orchestrator.rs
├── ipc.rs
├── errors.rs          # [NEW] Global error types
│
├── engines/
│   ├── mod.rs         # pub mod parser; pub mod confidence; etc.
│   ├── parser.rs
│   ├── confidence.rs
│   ├── types.rs
│   ├── semantic.rs
│   ├── intent.rs
│   ├── temporal.rs
│   └── propagation.rs
│
├── lsp/
│   ├── mod.rs
│   ├── client.rs
│   └── protocol.rs
│
├── data/
│   ├── mod.rs
│   ├── confidence_db.rs
│   ├── graph_store.rs
│   ├── embedding_index.rs
│   └── git_analysis.rs
│
└── models/
    ├── mod.rs
    ├── confidence.rs
    ├── type_dist.rs
    ├── propagation.rs
    └── embedding.rs
```

**`errors.rs`** — exact content:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdeError {
    #[error("Parser error: {0}")]
    Parser(String),
    #[error("Confidence engine error: {0}")]
    Confidence(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("LSP error: {0}")]
    Lsp(String),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type IdeResult<T> = Result<T, IdeError>;
```

**Acceptance**:
- [ ] `cargo check` succeeds with 0 errors, 0 warnings
- [ ] Every `mod.rs` re-exports its children
- [ ] `main.rs` has `mod orchestrator; mod ipc; mod errors; mod engines; mod lsp; mod data; mod models;`

---

### TASK 1.3: Define All Core Data Models

**What**: Implement every struct and enum in `models/`. These are shared across all engines.

**File `models/confidence.rs`** — exact content:
```rust
use serde::{Deserialize, Serialize};

/// Confidence score for a single line of code.
/// Invariant: 0.0 <= score <= 1.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub line: usize,
    pub score: f64,
    pub signals: Vec<Signal>,
}

impl ConfidenceScore {
    pub fn new(line: usize, score: f64, signals: Vec<Signal>) -> Self {
        let clamped = score.clamp(0.0, 1.0);
        Self { line, score: clamped, signals }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub name: SignalKind,
    pub value: f64,     // 0.0 to 1.0
    pub weight: f64,    // 0.0 to 1.0, sum of all weights = 1.0
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

/// Batch update sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceUpdate {
    pub file_path: String,
    pub scores: Vec<ConfidenceScore>,
}
```

**File `models/type_dist.rs`** — exact content:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDistribution {
    pub variable: String,
    pub location: Location,
    pub entries: Vec<TypeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeEntry {
    pub type_name: String,
    pub probability: f64,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub location: Location,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceKind {
    TypeGuard,
    Assignment,
    ReturnType,
    InstanceOf,
    TruthinessCheck,
    Comparison,
    FunctionSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}
```

**File `models/propagation.rs`** — exact content:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    pub source: FunctionId,
    pub affected: Vec<RippleNode>,
    pub total_files_affected: usize,
    pub total_functions_affected: usize,
    pub highest_risk: Option<RippleNode>,
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
    /// Returns the propagation weight for this edge type.
    /// These weights are fixed and defined in the spec.
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
```

**File `models/embedding.rs`** — exact content:
```rust
use serde::{Deserialize, Serialize};

pub const EMBEDDING_DIM: usize = 384;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEmbedding {
    pub function_id: super::propagation::FunctionId,
    pub vector: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub query: super::propagation::FunctionId,
    pub target: super::propagation::FunctionId,
    pub score: f64,
    pub classification: SimilarityClass,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SimilarityClass {
    Duplicate,       // > 0.95
    NearDuplicate,   // > 0.85
    Related,         // > 0.70
    Distinct,        // <= 0.70
}

impl SimilarityClass {
    pub fn from_score(score: f64) -> Self {
        if score > 0.95 { Self::Duplicate }
        else if score > 0.85 { Self::NearDuplicate }
        else if score > 0.70 { Self::Related }
        else { Self::Distinct }
    }
}
```

**Acceptance**:
- [ ] `cargo check` succeeds
- [ ] `cargo test --workspace` succeeds
- [ ] Each struct has `Debug, Clone, Serialize, Deserialize`
- [ ] `ConfidenceScore::new()` clamps score to [0.0, 1.0]
- [ ] `EdgeKind::weight()` returns exact values from spec
- [ ] `SimilarityClass::from_score()` matches exact thresholds

**Tests to write** (in each file, add `#[cfg(test)] mod tests {...}`):
```rust
// models/confidence.rs
#[test]
fn confidence_clamps_above_one() {
    let s = ConfidenceScore::new(1, 1.5, vec![]);
    assert_eq!(s.score, 1.0);
}
#[test]
fn confidence_clamps_below_zero() {
    let s = ConfidenceScore::new(1, -0.3, vec![]);
    assert_eq!(s.score, 0.0);
}

// models/propagation.rs
#[test]
fn edge_weights_are_correct() {
    assert_eq!(EdgeKind::DirectCall.weight(), 0.50);
    assert_eq!(EdgeKind::TestCoverage.weight(), 0.80);
    assert_eq!(EdgeKind::ImportCrossModule.weight(), 0.20);
}

// models/embedding.rs
#[test]
fn similarity_thresholds() {
    assert_eq!(SimilarityClass::from_score(0.96), SimilarityClass::Duplicate);
    assert_eq!(SimilarityClass::from_score(0.90), SimilarityClass::NearDuplicate);
    assert_eq!(SimilarityClass::from_score(0.75), SimilarityClass::Related);
    assert_eq!(SimilarityClass::from_score(0.50), SimilarityClass::Distinct);
}
```

---

### TASK 1.4: Implement Parser Engine (Tree-sitter Integration)

**File**: `engines/parser.rs`

**Public API** (implement exactly these signatures):
```rust
pub struct ParserEngine {
    parsers: HashMap<String, tree_sitter::Parser>,
}

impl ParserEngine {
    pub fn new() -> IdeResult<Self>;
    pub fn parse(&mut self, source: &str, lang: &str) -> IdeResult<tree_sitter::Tree>;
    pub fn parse_incremental(
        &mut self,
        source: &str,
        lang: &str,
        old_tree: &tree_sitter::Tree,
        edit: &tree_sitter::InputEdit,
    ) -> IdeResult<tree_sitter::Tree>;
    pub fn detect_language(file_path: &str) -> &'static str;
    pub fn has_syntax_errors(tree: &tree_sitter::Tree) -> bool;
    pub fn get_changed_ranges(
        old_tree: &tree_sitter::Tree,
        new_tree: &tree_sitter::Tree,
    ) -> Vec<tree_sitter::Range>;
}
```

**Language Detection** (exact mapping):
```rust
fn detect_language(file_path: &str) -> &'static str {
    match file_path.rsplit('.').next() {
        Some("ts" | "tsx") => "typescript",
        Some("js" | "jsx") => "javascript",
        Some("py") => "python",
        Some("rs") => "rust",
        _ => "plaintext",
    }
}
```

**Critical rules**:
- `new()` initializes parsers for ALL 4 P0 languages at startup
- `parse()` must complete in < 10ms for files under 10,000 lines
- `parse_incremental()` must complete in < 1ms
- `has_syntax_errors()` walks the tree checking `node.is_error()`

**Tests**:
```rust
#[test]
fn parse_valid_typescript() {
    let mut engine = ParserEngine::new().unwrap();
    let tree = engine.parse("const x: number = 5;", "typescript").unwrap();
    assert!(!ParserEngine::has_syntax_errors(&tree));
}

#[test]
fn parse_invalid_syntax_detected() {
    let mut engine = ParserEngine::new().unwrap();
    let tree = engine.parse("const x: = ;", "typescript").unwrap();
    assert!(ParserEngine::has_syntax_errors(&tree));
}

#[test]
fn detect_lang_from_extension() {
    assert_eq!(ParserEngine::detect_language("foo.ts"), "typescript");
    assert_eq!(ParserEngine::detect_language("bar.py"), "python");
    assert_eq!(ParserEngine::detect_language("baz.rs"), "rust");
    assert_eq!(ParserEngine::detect_language("file.unknown"), "plaintext");
}
```

---

### TASK 1.5: Implement Confidence DB (SQLite Persistence)

**File**: `data/confidence_db.rs`

**Schema** (execute on DB init):
```sql
CREATE TABLE IF NOT EXISTS confidence_scores (
    file_path TEXT NOT NULL,
    line INTEGER NOT NULL,
    score REAL NOT NULL CHECK(score >= 0.0 AND score <= 1.0),
    signals_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (file_path, line)
);

CREATE TABLE IF NOT EXISTS file_scores (
    file_path TEXT PRIMARY KEY,
    avg_score REAL NOT NULL CHECK(avg_score >= 0.0 AND avg_score <= 1.0),
    min_score REAL NOT NULL,
    line_count INTEGER NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_scores_file ON confidence_scores(file_path);
CREATE INDEX IF NOT EXISTS idx_file_scores_avg ON file_scores(avg_score);
```

**Public API**:
```rust
pub struct ConfidenceDb {
    conn: rusqlite::Connection,
}

impl ConfidenceDb {
    pub fn open(db_path: &std::path::Path) -> IdeResult<Self>;
    pub fn upsert_line_scores(&self, file: &str, scores: &[ConfidenceScore]) -> IdeResult<()>;
    pub fn get_file_scores(&self, file: &str) -> IdeResult<Vec<ConfidenceScore>>;
    pub fn get_file_summary(&self, file: &str) -> IdeResult<Option<(f64, f64, usize)>>;
    pub fn delete_file(&self, file: &str) -> IdeResult<()>;
    pub fn get_all_file_summaries(&self) -> IdeResult<Vec<(String, f64, f64, usize)>>;
}
```

**Critical rules**:
- `open()` creates the database and runs the schema if it doesn't exist
- `upsert_line_scores()` uses `INSERT OR REPLACE` — never duplicates
- All writes use a transaction
- `get_file_summary()` returns `(avg_score, min_score, line_count)`

**Tests**:
```rust
#[test]
fn round_trip_scores() {
    let db = ConfidenceDb::open(Path::new(":memory:")).unwrap();
    let scores = vec![
        ConfidenceScore::new(1, 0.95, vec![]),
        ConfidenceScore::new(2, 0.40, vec![]),
    ];
    db.upsert_line_scores("test.ts", &scores).unwrap();
    let loaded = db.get_file_scores("test.ts").unwrap();
    assert_eq!(loaded.len(), 2);
    assert!((loaded[0].score - 0.95).abs() < f64::EPSILON);
}

#[test]
fn upsert_overwrites_existing() {
    let db = ConfidenceDb::open(Path::new(":memory:")).unwrap();
    db.upsert_line_scores("f.ts", &[ConfidenceScore::new(1, 0.5, vec![])]).unwrap();
    db.upsert_line_scores("f.ts", &[ConfidenceScore::new(1, 0.9, vec![])]).unwrap();
    let loaded = db.get_file_scores("f.ts").unwrap();
    assert_eq!(loaded.len(), 1);
    assert!((loaded[0].score - 0.9).abs() < f64::EPSILON);
}
```

---

### TASK 1.6: Implement Confidence Engine v1 (Weighted Sum)

**File**: `engines/confidence.rs`

**The Formula** (implement exactly):
```
C(line) = Σ(wᵢ × sᵢ(line))  where Σ(wᵢ) = 1.0

Default weights:
  syntax_valid:       0.20
  type_check_pass:    0.15
  lint_clean:         0.10
  test_covers:        0.20
  test_passes:        0.10
  ml_bug_probability: 0.15
  code_age_factor:    0.05
  author_experience:  0.05
```

**Weight Redistribution Rule**: If a signal is unavailable (`available == false`), redistribute its weight proportionally among available signals.

```rust
pub struct ConfidenceEngine {
    default_weights: HashMap<SignalKind, f64>,
}

impl ConfidenceEngine {
    pub fn new() -> Self;
    pub fn compute_line_confidence(&self, signals: &[Signal]) -> f64;
    pub fn compute_file_confidence(&self, lines: &[ConfidenceScore]) -> f64;
}
```

**`compute_line_confidence` algorithm** (pseudocode, implement exactly):
```
1. Collect all signals where available == true
2. Sum their weights → total_available_weight
3. If total_available_weight == 0.0, return 0.5 (unknown)
4. For each available signal:
     adjusted_weight = signal.weight / total_available_weight
     contribution += adjusted_weight * signal.value
5. Return contribution.clamp(0.0, 1.0)
```

**`compute_file_confidence`**: CVaR at 10% — average of the worst 10% of line scores.
```
1. Sort line scores ascending
2. Take first ceil(len * 0.10) scores (at least 1)
3. Return their average
```

**Tests** (MANDATORY — these define correctness):
```rust
#[test]
fn all_signals_available() {
    let engine = ConfidenceEngine::new();
    let signals = vec![
        Signal { name: SignalKind::SyntaxValid, value: 1.0, weight: 0.20, available: true },
        Signal { name: SignalKind::TypeCheckPass, value: 1.0, weight: 0.15, available: true },
        Signal { name: SignalKind::LintClean, value: 1.0, weight: 0.10, available: true },
        Signal { name: SignalKind::TestCovers, value: 1.0, weight: 0.20, available: true },
        Signal { name: SignalKind::TestPasses, value: 1.0, weight: 0.10, available: true },
        Signal { name: SignalKind::MlBugProbability, value: 1.0, weight: 0.15, available: true },
        Signal { name: SignalKind::CodeAgeFactor, value: 1.0, weight: 0.05, available: true },
        Signal { name: SignalKind::AuthorExperience, value: 1.0, weight: 0.05, available: true },
    ];
    let score = engine.compute_line_confidence(&signals);
    assert!((score - 1.0).abs() < 1e-10);
}

#[test]
fn weight_redistribution_when_unavailable() {
    let engine = ConfidenceEngine::new();
    // Only syntax (0.20) and lint (0.10) available, both value=1.0
    // Redistributed: syntax = 0.20/0.30 = 0.667, lint = 0.10/0.30 = 0.333
    // Score = 0.667*1.0 + 0.333*1.0 = 1.0
    let signals = vec![
        Signal { name: SignalKind::SyntaxValid, value: 1.0, weight: 0.20, available: true },
        Signal { name: SignalKind::LintClean, value: 1.0, weight: 0.10, available: true },
        Signal { name: SignalKind::TestCovers, value: 0.0, weight: 0.20, available: false },
    ];
    let score = engine.compute_line_confidence(&signals);
    assert!((score - 1.0).abs() < 1e-10);
}

#[test]
fn no_signals_returns_0_5() {
    let engine = ConfidenceEngine::new();
    let score = engine.compute_line_confidence(&[]);
    assert!((score - 0.5).abs() < 1e-10);
}

#[test]
fn cvar_file_confidence() {
    let engine = ConfidenceEngine::new();
    // 10 lines, scores: 0.1, 0.2, 0.3, ..., 1.0
    // Worst 10% = 1 line = 0.1
    let lines: Vec<ConfidenceScore> = (1..=10)
        .map(|i| ConfidenceScore::new(i, i as f64 * 0.1, vec![]))
        .collect();
    let file_score = engine.compute_file_confidence(&lines);
    assert!((file_score - 0.1).abs() < 1e-10);
}
```

---

### TASK 1.7: Implement Graph Store (Dependency Graph)

**File**: `data/graph_store.rs`

Uses `petgraph::DiGraph` internally.

```rust
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub struct GraphStore {
    graph: DiGraph<FunctionId, EdgeKind>,
    index: HashMap<FunctionId, NodeIndex>,
}

impl GraphStore {
    pub fn new() -> Self;
    pub fn add_function(&mut self, id: FunctionId) -> NodeIndex;
    pub fn add_edge(&mut self, from: &FunctionId, to: &FunctionId, kind: EdgeKind) -> IdeResult<()>;
    pub fn get_dependents(&self, id: &FunctionId) -> Vec<(FunctionId, EdgeKind)>;
    pub fn get_dependencies(&self, id: &FunctionId) -> Vec<(FunctionId, EdgeKind)>;
    pub fn remove_function(&mut self, id: &FunctionId);
    pub fn clear(&mut self);
    pub fn node_count(&self) -> usize;
    pub fn edge_count(&self) -> usize;
}
```

**Critical rules**:
- `add_function()` is idempotent — adding the same FunctionId twice returns the existing NodeIndex
- `add_edge()` returns `Err` if either node doesn't exist
- `get_dependents()` returns nodes that DEPEND ON the given function (reverse edges)
- `get_dependencies()` returns nodes that the given function DEPENDS ON (forward edges)

**Tests**:
```rust
#[test]
fn add_function_idempotent() {
    let mut store = GraphStore::new();
    let id = FunctionId { file: "a.ts".into(), name: "foo".into(), line: 1 };
    let n1 = store.add_function(id.clone());
    let n2 = store.add_function(id);
    assert_eq!(n1, n2);
    assert_eq!(store.node_count(), 1);
}

#[test]
fn dependents_found() {
    let mut store = GraphStore::new();
    let a = FunctionId { file: "a.ts".into(), name: "a".into(), line: 1 };
    let b = FunctionId { file: "b.ts".into(), name: "b".into(), line: 1 };
    store.add_function(a.clone());
    store.add_function(b.clone());
    store.add_edge(&a, &b, EdgeKind::DirectCall).unwrap();
    // b depends on a, so a's dependents should include b
    let deps = store.get_dependents(&a);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].0, b);
}
```

---

### TASK 1.8: Implement Propagation Engine

**File**: `engines/propagation.rs`

**Algorithm** (implement exactly):
```
propagate(source_id, delta, graph, current_scores):
  1. Create result list
  2. BFS from source_id through dependents
  3. For each dependent at depth d:
       edge_weight = EdgeKind.weight()
       propagated_delta = parent_delta * edge_weight
       if |propagated_delta| < 0.01: skip (damping threshold)
       new_confidence = current_confidence * (1 - |propagated_delta|)
       add RippleNode to results
  4. Stop after depth 3 (hard limit)
  5. Sort results by |delta| descending
  6. Return PropagationResult
```

```rust
pub struct PropagationEngine;

impl PropagationEngine {
    pub fn propagate(
        source: &FunctionId,
        delta: f64,
        graph: &GraphStore,
        current_scores: &HashMap<FunctionId, f64>,
    ) -> PropagationResult;
}
```

**Damping**: Stop propagation when `|propagated_delta| < 0.01`.
**Max depth**: 3 hops maximum.

**Tests**:
```rust
#[test]
fn propagation_attenuates() {
    // A calls B calls C. Delta at A = -0.30
    // B delta = -0.30 * 0.50 = -0.15
    // C delta = -0.15 * 0.50 = -0.075
    let mut graph = GraphStore::new();
    let a = FunctionId { file: "a.ts".into(), name: "a".into(), line: 1 };
    let b = FunctionId { file: "b.ts".into(), name: "b".into(), line: 1 };
    let c = FunctionId { file: "c.ts".into(), name: "c".into(), line: 1 };
    graph.add_function(a.clone());
    graph.add_function(b.clone());
    graph.add_function(c.clone());
    graph.add_edge(&a, &b, EdgeKind::DirectCall).unwrap();
    graph.add_edge(&b, &c, EdgeKind::DirectCall).unwrap();

    let mut scores = HashMap::new();
    scores.insert(a.clone(), 0.95);
    scores.insert(b.clone(), 0.88);
    scores.insert(c.clone(), 0.82);

    let result = PropagationEngine::propagate(&a, -0.30, &graph, &scores);
    assert_eq!(result.affected.len(), 2);
    // B: delta should be approx -0.15
    assert!((result.affected[0].delta - (-0.15)).abs() < 0.01);
}

#[test]
fn propagation_stops_below_threshold() {
    // With very small delta, propagation should produce empty results
    let graph = GraphStore::new();
    let a = FunctionId { file: "a.ts".into(), name: "a".into(), line: 1 };
    let scores = HashMap::new();
    let result = PropagationEngine::propagate(&a, -0.005, &graph, &scores);
    assert!(result.affected.is_empty());
}
```

---

### TASK 1.9: Implement Temporal Engine (Git Analysis)

**File**: `engines/temporal.rs` + `data/git_analysis.rs`

**Decay formula** (implement exactly):
```
C_temporal(file, t) = C_initial × e^(-λt)

λ = 0.1 × author_count + 0.05 × patch_frequency + 0.03 × churn_ratio
t = days since last meaningful commit to this file

author_count = number of distinct committers to this file (git blame)
patch_frequency = commits per month to this file (git log)
churn_ratio = lines_changed / total_lines over last 90 days
```

```rust
// data/git_analysis.rs
pub struct GitAnalysis {
    repo: git2::Repository,
}

impl GitAnalysis {
    pub fn open(repo_path: &std::path::Path) -> IdeResult<Self>;
    pub fn author_count(&self, file_path: &str) -> IdeResult<usize>;
    pub fn patch_frequency(&self, file_path: &str, days: u32) -> IdeResult<f64>;
    pub fn churn_ratio(&self, file_path: &str, days: u32) -> IdeResult<f64>;
    pub fn days_since_last_commit(&self, file_path: &str) -> IdeResult<f64>;
}

// engines/temporal.rs
pub struct TemporalEngine;

impl TemporalEngine {
    pub fn compute_decay(
        author_count: usize,
        patch_frequency: f64,
        churn_ratio: f64,
        days_since_commit: f64,
        initial_confidence: f64,
    ) -> f64 {
        let lambda = 0.1 * author_count as f64
            + 0.05 * patch_frequency
            + 0.03 * churn_ratio;
        let decayed = initial_confidence * (-lambda * days_since_commit / 365.0).exp();
        decayed.clamp(0.0, 1.0)
    }
}
```

**Tests**:
```rust
#[test]
fn no_decay_for_recent_code() {
    let score = TemporalEngine::compute_decay(1, 0.5, 0.1, 0.0, 0.95);
    assert!((score - 0.95).abs() < 1e-10); // e^0 = 1
}

#[test]
fn high_decay_for_abandoned_code() {
    // 3 authors, 5 patches/month, 0.8 churn, 365 days
    // λ = 0.3 + 0.25 + 0.024 = 0.574
    // C = 0.95 * e^(-0.574) ≈ 0.534
    let score = TemporalEngine::compute_decay(3, 5.0, 0.8, 365.0, 0.95);
    assert!(score < 0.60);
    assert!(score > 0.45);
}
```

---

### TASK 1.10: Implement Orchestrator (Event Router)

**File**: `orchestrator.rs`

The orchestrator is the central hub. It receives editor events and dispatches to engines.

```rust
pub struct Orchestrator {
    parser: ParserEngine,
    confidence: ConfidenceEngine,
    propagation_engine: PropagationEngine,
    graph: GraphStore,
    db: ConfidenceDb,
}

impl Orchestrator {
    pub fn new(project_path: &std::path::Path) -> IdeResult<Self>;

    /// Called on every keystroke. Must complete in < 16ms.
    pub fn on_keystroke(
        &mut self,
        file_path: &str,
        source: &str,
        lang: &str,
    ) -> IdeResult<ConfidenceUpdate>;

    /// Called on file save. Can take up to 500ms.
    pub fn on_file_save(
        &mut self,
        file_path: &str,
        source: &str,
        lang: &str,
    ) -> IdeResult<(ConfidenceUpdate, PropagationResult)>;

    /// Called on project open. Background, can take 5-30s.
    pub fn on_project_open(&mut self) -> IdeResult<Vec<(String, f64)>>;
}
```

**`on_keystroke` pipeline**:
1. Parse incrementally (< 1ms)
2. Check `has_syntax_errors` → set `SyntaxValid` signal
3. Compute confidence for changed lines only
4. Return `ConfidenceUpdate`

**`on_file_save` pipeline**:
1. Full parse
2. Compute confidence for all lines
3. Persist to SQLite
4. Run propagation from changed functions
5. Return both updates

---

### TASK 1.11: Implement IPC Bridge (Tauri Commands)

**File**: `ipc.rs`

```rust
use tauri::State;

#[tauri::command]
pub fn open_file(
    path: String,
    orchestrator: State<'_, std::sync::Mutex<Orchestrator>>,
) -> Result<ConfidenceUpdate, String>;

#[tauri::command]
pub fn on_keystroke(
    path: String,
    source: String,
    lang: String,
    orchestrator: State<'_, std::sync::Mutex<Orchestrator>>,
) -> Result<ConfidenceUpdate, String>;

#[tauri::command]
pub fn on_save(
    path: String,
    source: String,
    lang: String,
    orchestrator: State<'_, std::sync::Mutex<Orchestrator>>,
) -> Result<serde_json::Value, String>;

#[tauri::command]
pub fn get_file_confidence(
    path: String,
    orchestrator: State<'_, std::sync::Mutex<Orchestrator>>,
) -> Result<Vec<ConfidenceScore>, String>;
```

Register in `main.rs`:
```rust
fn main() {
    tauri::Builder::default()
        .manage(std::sync::Mutex::new(
            Orchestrator::new(std::path::Path::new(".")).expect("Failed to init orchestrator")
        ))
        .invoke_handler(tauri::generate_handler![
            ipc::open_file,
            ipc::on_keystroke,
            ipc::on_save,
            ipc::get_file_confidence,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## MILESTONE 2 — "The Gutter" (Frontend)

### TASK 2.1: Install CodeMirror 6 Editor

**Install**:
```bash
npm install @codemirror/view @codemirror/state @codemirror/lang-javascript @codemirror/lang-python @codemirror/lang-rust @codemirror/theme-one-dark
```

**File**: `src/editor/Editor.ts`

Create a CodeMirror 6 instance with:
- Extensions: syntax highlighting, line numbers, bracket matching, `oneDark` theme
- Event listener: on every doc change, call `invoke('on_keystroke', { path, source, lang })`
- On save (Ctrl+S): call `invoke('on_save', { path, source, lang })`

### TASK 2.2: Implement Confidence Gutter Renderer

**File**: `src/editor/ConfidenceGutter.ts`

Use CodeMirror 6's `GutterMarker` API.

**Color Scale** (exact HSL values):
```typescript
function scoreToColor(score: number): string {
    if (score >= 0.9) return 'hsl(140, 80%, 50%)';   // deep green
    if (score >= 0.7) return 'hsl(120, 60%, 55%)';   // green
    if (score >= 0.5) return 'hsl(60, 70%, 50%)';    // yellow-green
    if (score >= 0.3) return 'hsl(40, 80%, 50%)';    // amber
    if (score >= 0.1) return 'hsl(20, 90%, 50%)';    // orange
    return 'hsl(0, 100%, 50%)';                       // red
}
```

**Gutter bar**: 6px wide, 2px gap from line number. Rendered as a `<div>` with `background-color` set by `scoreToColor()`.

### TASK 2.3: Implement Type Distribution Tooltip

**File**: `src/editor/TypeTooltip.ts`

On hover over a variable, call `invoke('get_type_distribution', { path, line, column })` and render:
- Variable name
- Bar chart showing each type's probability (horizontal bars)
- Evidence list below
- Suggestion text (if any)

### TASK 2.4: Implement IPC Bridge (TypeScript side)

**File**: `src/ipc/bridge.ts`

```typescript
import { invoke } from '@tauri-apps/api/core';

export async function openFile(path: string): Promise<ConfidenceUpdate> {
    return invoke('open_file', { path });
}

export async function onKeystroke(path: string, source: string, lang: string): Promise<ConfidenceUpdate> {
    return invoke('on_keystroke', { path, source, lang });
}

export async function onSave(path: string, source: string, lang: string): Promise<SaveResult> {
    return invoke('on_save', { path, source, lang });
}
```

**File**: `src/ipc/events.ts`

```typescript
export interface ConfidenceScore {
    line: number;
    score: number;
    signals: Signal[];
}

export interface ConfidenceUpdate {
    file_path: string;
    scores: ConfidenceScore[];
}

export interface Signal {
    name: string;
    value: number;
    weight: number;
    available: boolean;
}
```

---

## MILESTONE 3+ — Remaining Task Headers

> The following are task headers for later milestones. Each follows the same exhaustive pattern as above.

### TASK 3.1: ML Inference Server (Python FastAPI)
- File: `ml/server.py`
- Endpoints: `POST /embed`, `POST /predict-bugs`, `POST /analyze-intent`
- Model: `all-MiniLM-L6-v2` via `sentence-transformers` + ONNX
- Tests: `ml/test_server.py`

### TASK 3.2: Integrate ONNX Runtime in Rust
- Crate: `ort = "2"` (official ONNX Runtime Rust bindings)
- File: `engines/semantic.rs`
- Calls ML server over HTTP at `http://localhost:8765`

### TASK 3.3: Bug Predictor (XGBoost)
- File: `ml/bug_predictor.py`
- Input features: cyclomatic complexity, AST depth, author count, churn, LOC
- Output: `P(bug)` float [0, 1]
- Model file: `ml/models/bug_predictor.xgb`

### TASK 3.4: Probabilistic Type Engine
- File: `engines/types.rs`
- Implements Bayes' rule: `P(T|E) = P(E|T) × P(T) / P(E)`
- Data structures: `TypeDistribution`, `TypeEntry`, `Evidence` (already defined in models)

### TASK 3.5: Intent Engine (Comment ↔ Code Alignment)
- File: `engines/intent.rs`
- Cosine similarity between comment embedding and code embedding
- Threshold: < 0.5 alignment = "DIVERGENT", flag to developer

### TASK 3.6: Semantic Engine (HNSW Vector Index)
- File: `engines/semantic.rs` + `data/embedding_index.rs`
- Uses `hnswlib` or `instant-distance` crate for approximate nearest neighbor
- Clustering thresholds: >0.95 DUPLICATE, >0.85 NEAR-DUPLICATE, >0.70 RELATED

### TASK 4.1: Propagation Ripple View (Frontend)
- File: `src/editor/PropagationPanel.ts`
- Tree view showing affected functions with Δ values
- Click to navigate to affected file/line

### TASK 4.2: Semantic Map (WebGL 2D View)
- File: `src/views/SemanticMap.ts`
- Renders function clusters as colored circles
- Size = LOC, Color = confidence, Position = embedding reduced to 2D via UMAP

### TASK 5.1: Comprehension Dashboard
- File: `src/views/Dashboard.ts`
- Shows project-wide confidence, top risks, module scores

### TASK 5.2: Code Heartbeat Visualization
- File: `src/views/Heartbeat.ts`
- Line chart showing confidence over time per file (from SQLite history)

---

## 🔒 Final Verification Checklist

After ALL tasks are complete, run these commands:

```bash
# 1. Rust: zero errors, zero warnings
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace

# 2. Frontend: zero errors
npm run build
npm run lint

# 3. ML: tests pass
cd ml && python -m pytest -v

# 4. Full integration: app launches
cargo tauri dev
```

**Every single command above MUST exit with code 0.**
