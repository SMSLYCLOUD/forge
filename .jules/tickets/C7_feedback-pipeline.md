# TICKET C7 — Closed-Loop Feedback Pipeline

## Context
- Source: Sessions 14, 17
- Developer actions feed back into confidence model via EMA (α=0.1)
- Converges after ~50 actions to personalized model

## Requirements
1. Track developer actions: ignore warning, fix flagged line, dismiss suggestion, add test, commit low-C code
2. Each action updates prior via exponential moving average: `prior_new = α × evidence + (1-α) × prior_old`
3. Per-module priors (developer might know module A well, not module B)
4. Store priors in local JSON file (privacy-first)
5. After ~50 actions, model is personalized

## Files to Create
- `forge/crates/forge-feedback/src/lib.rs`
- `forge/crates/forge-feedback/src/tracker.rs` — action tracking
- `forge/crates/forge-feedback/src/ema.rs` — EMA update
- `forge/crates/forge-feedback/src/store.rs` — JSON persistence
- `forge/crates/forge-feedback/Cargo.toml`
- `forge/crates/forge-feedback/tests/test_feedback.rs`

## Acceptance Criteria
- [ ] Actions tracked correctly
- [ ] EMA updates prior as expected
- [ ] Persistence works (priors survive restart)
- [ ] After 50 simulated actions, model diverges from default
- [ ] Privacy: all data local, deletable

## Dependencies: C1
## Effort: 2 days → WITH JULES: ~1 session
