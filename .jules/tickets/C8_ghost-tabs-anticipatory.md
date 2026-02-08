# TICKET C8 — Ghost Tabs + Anticipatory Workspace

## Context
- Source: Sessions 14b, 17
- Discovery: `discoveries/2026-02-08_sub-binary-ide-anticipatory-living-workspace.md`
- Ghost tabs = pre-loaded files you'll probably need (Markov chain)

## Requirements

### Ghost Tabs
1. Track file open history as sequence
2. Build first-order Markov chain: P(next_file | current_file)
3. When user opens file A, predict top-3 next files
4. Pre-load as "ghost tabs" (semi-transparent, not focused)
5. If user opens a ghost tab → promote to real tab
6. Threshold: P > 0.3 to show ghost tab
7. Storage: ~10KB per project (transition matrix)

### Branch Workspace Snapshots
1. On branch switch, save: open files, cursor positions, scroll positions, split layout
2. On branch return, restore entire workspace state
3. JSON serialization

## Files to Create
- `forge/crates/forge-anticipation/src/lib.rs`
- `forge/crates/forge-anticipation/src/ghost_tabs.rs`
- `forge/crates/forge-anticipation/src/markov.rs`
- `forge/crates/forge-anticipation/src/workspace_snapshot.rs`
- `forge/crates/forge-anticipation/Cargo.toml`
- `forge/crates/forge-anticipation/tests/test_ghost_tabs.rs`
- `forge/crates/forge-anticipation/tests/test_snapshots.rs`

## Acceptance Criteria
- [ ] Markov chain built from file history
- [ ] Ghost tabs predicted with P > 0.3
- [ ] Workspace snapshots save/restore correctly
- [ ] Storage < 50KB per project
- [ ] Tests pass

## Dependencies: C6
## Effort: 3 days → WITH JULES: ~1 session
