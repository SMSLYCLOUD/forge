# TICKET C1 — Confidence Gutter (Forge/Rust)

## Context
- Source: Session 9, 17
- Discovery: `discoveries/2026-02-08_sub-binary-ide-architecture.md`
- Mega-synthesis: `discoveries/2026-02-08_mega-synthesis-sub-binary-ide.md`
- Replaces binary red/green squiggles with continuous confidence heatmap

## Requirements
Implement confidence gutter as a Rust library crate:

### Data Structures
```rust
pub struct ConfidenceScore {
    pub overall: f64,           // 0.0 to 1.0
    pub criteria: CriteriaBreakdown,
    pub sources: Vec<EvidenceSource>,
}

pub struct CriteriaBreakdown {
    pub syntax: f64,     // 1.0 if parseable (proven)
    pub type_safety: f64, // 1.0 if type-checked (proven)
    pub lint: f64,        // 1.0 if lint-clean (proven)
    pub runtime: f64,     // 0.0-1.0 (estimated)
    pub behavior: f64,    // 0.0-1.0 (estimated)
    pub security: f64,    // 0.0-1.0 (estimated)
}

pub struct LineConfidence {
    pub line: usize,
    pub score: ConfidenceScore,
    pub color: RgbaColor,  // Gradient from red to green
}
```

### Functions
- `compute_line_confidence(file: &ParsedFile) -> Vec<LineConfidence>`
- `color_from_confidence(c: f64) -> RgbaColor` — smooth gradient
- `aggregate_file_confidence(lines: &[LineConfidence]) -> ConfidenceScore`

### Initial Scoring (v1)
- K_syntax = 1.0 if Tree-sitter parses without error, 0.0 otherwise
- K_type = 1.0 if language server reports no type errors, 0.0 otherwise  
- K_lint = 1.0 if no lint warnings on this line, 0.9 if warnings, 0.0 if errors
- K_runtime = 0.5 (default, improved by later modules)
- K_behavior = 0.5 (default)
- K_security = 0.5 (default)
- Overall = CVaR_0.95(K_syntax, K_type, K_lint, K_runtime, K_behavior, K_security)

## Files to Create
- `forge/crates/forge-confidence/src/lib.rs`
- `forge/crates/forge-confidence/src/score.rs`
- `forge/crates/forge-confidence/src/color.rs`
- `forge/crates/forge-confidence/src/aggregate.rs`
- `forge/crates/forge-confidence/Cargo.toml`
- `forge/crates/forge-confidence/tests/test_scoring.rs`

## Acceptance Criteria
- [ ] `cargo build` passes
- [ ] `cargo test` passes (10+ test cases)
- [ ] Confidence score correctly computed for sample files
- [ ] Color gradient is smooth and visually correct
- [ ] CVaR aggregation implemented correctly

## Effort: 5 days → WITH JULES: ~2 sessions
