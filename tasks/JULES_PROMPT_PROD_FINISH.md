# JULES PRODUCTION FINISH PROMPT (ZERO-ERROR HANDOFF)

You are finishing Forge to production readiness from the current `master` branch state.
Treat this as a release-hardening task, not exploratory work.

## Non-Negotiable Constraints

1. Do not rewrite architecture unless explicitly required by a blocker.
2. Keep all existing passing behavior and tests intact.
3. No `unwrap()`/`expect()` in production paths unless truly unreachable with a reason.
4. Every new public function/type requires doc comments and tests.
5. Preserve workspace dependency conventions.
6. Final result must pass:
   - `cargo fmt --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo check --workspace`

## Current Known Gaps You Must Close

Use these exact references as starting points:
- LSP runtime is still stubbed:
  - `crates/forge-app/src/application.rs:299`
  - `crates/forge-app/src/application.rs:303`
- AI runtime is still stubbed:
  - `crates/forge-app/src/application.rs:306`
  - `crates/forge-app/src/application.rs:309`
- Debug breakpoint sync is TODO:
  - `crates/forge-app/src/debug_ui.rs:52`
- Split editor command still not implemented:
  - `crates/forge-app/src/application.rs:1917`
- Terminal UI render path TODO:
  - `crates/forge-app/src/terminal_ui.rs:47`
- Extension registry/runtime still placeholder-level:
  - `crates/forge-app/src/extensions.rs:87`
  - `crates/forge-plugin/src/runtime.rs:31`
- Advanced modules are still commented out:
  - `crates/forge-app/src/main.rs:79`
  - `crates/forge-app/src/main.rs:81`
  - `crates/forge-app/src/main.rs:84`
- Cut operation still incomplete:
  - `crates/forge-app/src/application.rs:2032`

## Required Deliverables

### 1) Real LSP Integration in App Runtime

Implement actual LSP lifecycle wiring inside `forge-app`:
- Start rust-analyzer at app init when possible.
- Initialize transport and `initialize` request.
- Open active document (`didOpen`) for current file tab.
- Send `didChange` notifications after edits (throttled/debounced if needed, but reliable).
- Gracefully handle server spawn/initialize failure without crashing app.
- Add minimal logging for state transitions: spawned, initialized, failed, disconnected.

Definition of done:
- Opening/editing a Rust file triggers LSP open/change traffic.
- App remains stable if rust-analyzer is missing.

### 2) AI Runtime Integration

Wire `forge-agent` runtime into app lifecycle:
- Initialize agent runtime from config.
- Provide a non-blocking start path.
- Add clean fallback if initialization fails.
- Expose one real user-facing interaction (even minimal), not just stubs.

Definition of done:
- Agent runtime can start without freezing UI.
- Failure path is handled and visible via notification/log.

### 3) Debug Breakpoint Synchronization (DAP)

In debug UI/client integration:
- On toggle breakpoint, send/set DAP breakpoints request against active file.
- Keep local breakpoint model in sync with adapter responses.
- Handle adapter-not-running gracefully.

Definition of done:
- Breakpoint toggles are reflected in adapter traffic.
- No panic on adapter errors.

### 4) Split Editor Implementation

Implement the `Ctrl+\` path in `application.rs` using existing editor group primitives.
- At minimum: create second split and make both panes functional with independent cursor/scroll state.
- Keep layout resizing and redraw stable.
- Provide clear fallback notification if a sub-path is unsupported.

Definition of done:
- `Ctrl+\` no longer shows "not available yet".
- Two-pane editing works for open files.

### 5) Terminal UI Rendering Completion

Replace placeholder TODO path in `terminal_ui.rs` / app integration:
- Ensure terminal text rendering path is fully integrated with glyphon in production code path.
- Cursor + scroll behavior should be coherent.

Definition of done:
- Terminal panel renders live terminal output robustly.

### 6) Extension Runtime Hardening

Improve extension pipeline to move beyond placeholders:
- Parse plugin metadata/manifest and set real plugin names (replace `"unknown"`).
- Improve extension list source (no hardcoded-only behavior).
- Ensure extension load errors are isolated and observable.

Definition of done:
- Runtime can report real extension/plugin metadata.
- Extension subsystem does not fail app startup.

### 7) Enable Advanced Navigation Modules

Uncomment and integrate module paths in `main.rs` where feasible:
- `go_to_def`
- `references`
- `workspace_symbols`

If full enablement is blocked, implement complete vertical slices for at least one and document exact blockers for others.

Definition of done:
- At least one advanced symbol-navigation feature is fully wired and user-reachable.
- Remaining blockers, if any, are concrete and minimal.

### 8) Complete Cut-Line Behavior

Implement delete-after-copy for cut operation in `application.rs` (`Ctrl+X` flow).

Definition of done:
- Current line is copied and removed atomically.
- Undo/redo semantics remain correct.

## QA Matrix (Must Be Executed)

1. Build/Test:
   - `cargo fmt --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo check --workspace`

2. Manual Smoke:
   - Launch app with a Rust file.
   - Verify Find/Replace/GoToLine/Command Palette still work.
   - Verify terminal toggle and output.
   - Verify split editor behavior.
   - Verify breakpoint toggle path with debug adapter available and unavailable.
   - Verify app starts cleanly even when rust-analyzer is missing.

3. Regression Check:
   - No broken shortcuts from existing recent changes.
   - No startup panic in normal path.

## Output Format Required From You

Return a final report with:
1. `Implemented`:
   - Bullet list of completed items mapped to the deliverables above.
2. `Files Changed`:
   - Exact file paths.
3. `Validation`:
   - Command list and pass/fail.
4. `Known Remaining Gaps`:
   - Only if truly blocked; include exact reason and smallest next action.

Do not claim completion if any required validation command fails.
