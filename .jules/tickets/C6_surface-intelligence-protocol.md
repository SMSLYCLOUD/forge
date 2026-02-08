# TICKET C6 — Surface Intelligence Protocol + Intelligent File Explorer

## Context
- Source: Sessions 12, 17
- Discovery: `discoveries/2026-02-08_sub-binary-ide-pervasive-intelligence.md`
- SIP trait defined in mega-synthesis

## Requirements

### SIP Trait
```rust
pub trait SurfaceIntelligence {
    fn surface_id(&self) -> &str;
    fn information_cost(&self) -> f64;  // bits consumed from noise budget
    fn render(&self, confidence: &ConfidenceField, mode: ConfidenceMode) -> SurfaceState;
    fn priority(&self, context: &WorkspaceContext) -> f64;
}
```

### Intelligent File Explorer (first SIP implementation)
1. Sort files by confidence (worst-first = "what should I work on?")
2. Color-code file icons by confidence level
3. "Relevant Zone" — files related to current task float to top
4. Confidence badge on each file: red/yellow/green dot

## Files to Create
- `forge/crates/forge-surfaces/src/lib.rs`
- `forge/crates/forge-surfaces/src/protocol.rs` — SIP trait
- `forge/crates/forge-surfaces/src/file_explorer.rs` — intelligent explorer
- `forge/crates/forge-surfaces/Cargo.toml`
- `forge/crates/forge-surfaces/tests/test_explorer.rs`

## Acceptance Criteria
- [ ] SIP trait defined and documented
- [ ] File explorer implements SIP
- [ ] Files sorted by confidence (configurable)
- [ ] Color-coding works
- [ ] Tests pass

## Dependencies: C1
## Effort: 3 days → WITH JULES: ~1 session
