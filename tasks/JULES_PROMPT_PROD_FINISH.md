# FORGE PROD FINISH - STRICT ZERO-MISS EXECUTION PROMPT

You are finalizing Forge for production hardening from current `master`.
This is not a brainstorming task.
This is a release execution task with strict acceptance criteria.

If any deliverable is incomplete, you are not done.

## 0) Mission

Ship Forge in a polished, robust, low-regression state by removing known runtime stubs, wiring missing core behaviors, and validating the full workspace.

Primary goals:
1. Stability.
2. Functional completeness for currently exposed UX paths.
3. Predictable fallback behavior when external services are unavailable.
4. No hidden TODO behavior in critical code paths.

## 1) Hard Rules (Non-Negotiable)

1. Do not do broad rewrites when targeted fixes are sufficient.
2. Preserve existing passing behavior.
3. No new `unwrap()`/`expect()` in production paths unless truly unreachable and documented.
4. Every new public API must include doc comments and tests.
5. Keep dependency usage aligned with workspace conventions.
6. Do not leave dead stubs in critical execution paths without explicit graceful handling.
7. Do not claim success if any required validation command fails.

## 2) Required Validation Gates (Must all pass)

Run all of these at the end:
1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo check --workspace`

Also run targeted checks while implementing each tranche to prevent drift.

## 3) Baseline Known Gaps (Start Here)

Use these exact references first:
1. LSP init still stubbed:
   - `crates/forge-app/src/application.rs:299`
   - `crates/forge-app/src/application.rs:303`
2. AI runtime still stubbed:
   - `crates/forge-app/src/application.rs:306`
   - `crates/forge-app/src/application.rs:309`
3. Debug breakpoint sync TODO:
   - `crates/forge-app/src/debug_ui.rs:52`
4. Split editor not implemented:
   - `crates/forge-app/src/application.rs:1917`
5. Terminal UI render TODO:
   - `crates/forge-app/src/terminal_ui.rs:47`
6. Extension system placeholders:
   - `crates/forge-app/src/extensions.rs:87`
   - `crates/forge-plugin/src/runtime.rs:31`
7. Advanced navigation modules commented:
   - `crates/forge-app/src/main.rs:79`
   - `crates/forge-app/src/main.rs:81`
   - `crates/forge-app/src/main.rs:84`
8. Cut action still incomplete:
   - `crates/forge-app/src/application.rs:2032`

## 4) Execution Order (Mandatory)

Execute in this order to minimize regressions:
1. LSP runtime wiring.
2. AI runtime wiring.
3. Debug breakpoint adapter sync.
4. Split editor implementation.
5. Terminal rendering completion.
6. Extension runtime hardening.
7. Advanced symbol navigation integration.
8. Cut-line completion.
9. UX polish and error-path polish pass.
10. Full validation gates.

If blocked, continue with next item while documenting blocker with precise cause and smallest unblocking action.

## 5) Deliverable Requirements

### 5.1 LSP Runtime Integration (Production-grade)

Must implement:
1. Spawn `rust-analyzer` when appropriate at app startup.
2. Initialize transport and send `initialize`.
3. Register workspace folder correctly.
4. Send `didOpen` for active editor buffer.
5. Send `didChange` after edits, with reliable sequencing.
6. Optionally send `didSave` when file is saved.
7. Handle missing server executable cleanly.
8. Handle transport disconnect without crashing UI.
9. Keep logs clear and actionable.

Minimum acceptance:
1. Opening and typing in a Rust file emits LSP traffic.
2. Missing `rust-analyzer` produces graceful warning/notification, not crash.
3. App continues usable if LSP fails.

### 5.2 AI Runtime Integration (No freeze, no ghost stubs)

Must implement:
1. Initialize agent runtime from existing config.
2. Keep initialization asynchronous/non-blocking.
3. Add robust failure fallback.
4. Wire one real user-reachable action from UI to runtime.
5. Surface status in logs/notification.

Minimum acceptance:
1. Agent startup does not block render loop.
2. Failure path is visible and non-fatal.
3. At least one UI action triggers actual runtime behavior.

### 5.3 Debug Breakpoint Synchronization (DAP)

Must implement:
1. On breakpoint toggle, send corresponding DAP `setBreakpoints`.
2. Use active file context and line mapping correctly.
3. Keep local model synchronized to adapter response.
4. Handle no-adapter state and adapter errors gracefully.

Minimum acceptance:
1. Toggle emits adapter request.
2. No panic when adapter unavailable.
3. Local breakpoint list remains coherent after failures.

### 5.4 Split Editor Implementation (`Ctrl+\\`)

Must implement:
1. Replace fallback warning with actual split behavior.
2. Create second pane with independent cursor/scroll context.
3. Maintain redraw correctness on resize.
4. Keep tab/editor interactions stable.
5. If multi-pane ownership has architectural constraints, implement minimum viable reliable split with explicit limits.

Minimum acceptance:
1. `Ctrl+\\` creates usable second pane.
2. Both panes can navigate/edit without immediate corruption.

### 5.5 Terminal Rendering Completion

Must implement:
1. Remove placeholder TODO path in terminal UI flow.
2. Ensure terminal text render path is actually integrated with glyphon flow in app render path.
3. Keep cursor position and viewport coherent.
4. Avoid regressions in panel toggling.

Minimum acceptance:
1. Terminal panel displays live output in normal use.
2. Toggle open/close remains stable.

### 5.6 Extension Runtime Hardening

Must implement:
1. Replace `"unknown"` plugin naming with real metadata extraction.
2. Improve extension discovery/source beyond hardcoded-only list.
3. Validate extension load errors are isolated.
4. Keep startup resilient if extension load fails.

Minimum acceptance:
1. Runtime reports real plugin identity when available.
2. Extension failure does not crash app.

### 5.7 Advanced Navigation Modules

Must implement:
1. Integrate `go_to_def`, `references`, `workspace_symbols` where feasible.
2. Expose at least one feature end-to-end and user reachable.
3. If full integration blocked, provide concrete blockers and partial delivery with tests.

Minimum acceptance:
1. One advanced navigation feature fully functional.
2. Others either functional or precisely blocked with smallest next action.

### 5.8 Complete Cut-Line Behavior (`Ctrl+X`)

Must implement:
1. Copy current line to clipboard.
2. Delete that line in same flow.
3. Maintain undo/redo correctness.
4. Preserve cursor and scroll consistency.

Minimum acceptance:
1. Cut behaves as users expect on current line.
2. Undo restores exact prior state.

## 6) Reliability and Polish Requirements

Apply a dedicated polish pass after functional implementation:
1. Replace vague logs with actionable messages.
2. Ensure all new error paths include context.
3. Remove or convert stale TODO markers in touched code paths.
4. Avoid noisy notifications for routine background events.
5. Ensure keybindings do not conflict with existing behaviors.
6. Verify no accidental performance regressions in render/input loops.

## 7) Regression Checklist (Must explicitly verify)

1. Existing Find/Replace/GoToLine/Command Palette behavior still works.
2. Save/close/undo/redo still works.
3. Terminal toggle and rendering still works.
4. App startup still works with and without optional external binaries.
5. No panics in normal user flow.
6. Previously passing tests stay passing.

## 8) Manual QA Matrix (Must execute)

Run these manual checks and report pass/fail:
1. Start app with a Rust file.
2. Type in editor and confirm no freeze.
3. Trigger LSP path with Rust file open.
4. Save file and verify stability.
5. Toggle terminal and verify visible output.
6. Trigger split editor and use both panes.
7. Toggle breakpoint with adapter available.
8. Toggle breakpoint with adapter unavailable.
9. Use command palette commands touched by your changes.
10. Verify cut-line behavior and undo/redo.

## 9) Code Hygiene Requirements

1. Keep changes scoped to relevant files.
2. Add tests for new behavior whenever feasible.
3. Do not leave commented-out dead code unless explicitly justified.
4. Keep naming and style aligned with existing crate patterns.

## 10) Final Output Contract (Required format)

Your final report must include all sections below:

1. `Implemented`
   - Map each completed item to sections 5.1 to 5.8.
2. `Files Changed`
   - Exact file paths.
3. `Validation`
   - Commands run and pass/fail.
4. `Manual QA`
   - Matrix results, each line pass/fail.
5. `Risks`
   - Remaining technical risks, if any.
6. `Known Remaining Gaps`
   - Only if truly blocked, with exact blocker and smallest next action.

You are not done until all required validation gates are green and the report is complete.
