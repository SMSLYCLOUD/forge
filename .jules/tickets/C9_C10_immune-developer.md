# TICKET C9 — Confidence Immune System

## Context
- Source: Session 14, 17
- 5 attack vectors identified, 5 defenses designed
- Mutation testing integration is defense #1

## Requirements
1. **Mutation testing validator**: Before accepting C=1.0 from test coverage, verify tests kill mutations
2. **Feedback anomaly detection**: Flag if developer dismisses >80% of warnings (feedback poisoning)
3. **ML signal cap**: ML-derived confidence never exceeds 25% of total weight (deterministic veto)
4. **Temporal decay**: Force re-verification after configurable time (default: 30 days)
5. **Audit trail**: Log all confidence changes with Merkle-hashed chain

## Files to Create
- `forge/crates/forge-immune/src/lib.rs`
- `forge/crates/forge-immune/src/mutation_validator.rs`
- `forge/crates/forge-immune/src/anomaly_detector.rs`
- `forge/crates/forge-immune/src/ml_cap.rs`
- `forge/crates/forge-immune/src/temporal_decay.rs`
- `forge/crates/forge-immune/src/audit.rs`
- `forge/crates/forge-immune/Cargo.toml`
- `forge/crates/forge-immune/tests/test_immune.rs`

## Acceptance Criteria
- [ ] Mutation testing correctly downgrades C when tests don't kill mutations
- [ ] Anomaly detection flags suspicious patterns
- [ ] ML cap enforced
- [ ] Temporal decay works
- [ ] Audit trail is tamper-evident

## Dependencies: C2
## Effort: 3 days → WITH JULES: ~1 session

---

# TICKET C10 — Developer Confidence Model

## Context
- Source: Session 14, 17 — C(change) = C(code) × C(developer, module)

## Requirements
1. Compute C(developer, module) from 7 evidence signals:
   - Commit history in this module
   - Bug introduction rate
   - Review acceptance rate
   - Recency of last edit
   - Domain expertise score
   - Flow score (from feedback pipeline)
   - Fatigue indicator (time since break)
2. Bus factor computation: how many devs have C(dev, module) > 0.8?
3. Knowledge graph: who knows what?
4. All data local and opt-in (privacy-first)

## Files to Create
- `forge/crates/forge-developer/src/lib.rs`
- `forge/crates/forge-developer/src/model.rs`
- `forge/crates/forge-developer/src/bus_factor.rs`
- `forge/crates/forge-developer/src/knowledge_graph.rs`
- `forge/crates/forge-developer/Cargo.toml`
- `forge/crates/forge-developer/tests/test_developer.rs`

## Acceptance Criteria
- [ ] C(developer, module) computed correctly
- [ ] Bus factor calculated per module
- [ ] All data local and deletable
- [ ] Tests pass

## Dependencies: C7
## Effort: 3 days → WITH JULES: ~1 session
