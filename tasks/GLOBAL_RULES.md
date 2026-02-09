# Forge — Global Rules for ALL Agents

> **READ THIS FIRST.** Every agent must comply with these rules.

## Workspace Structure
```
forge/
├── Cargo.toml          # Workspace root — add new crates here
├── crates/             # All crates live here
│   ├── forge-core/     # Rope buffer, transactions, history, syntax, git, terminal
│   ├── forge-renderer/ # wgpu GPU rendering, text atlas, pipeline, viewport
│   ├── forge-window/   # winit windowing, event loop, input
│   ├── forge-app/      # Main application (APPLICATION.RS — Batch 9 only)
│   ├── forge-config/   # TOML configuration
│   ├── forge-theme/    # Color themes
│   ├── forge-input/    # Keyboard/mouse input
│   ├── forge-confidence/  # Code confidence scoring
│   ├── forge-propagation/ # Change propagation engine
│   ├── forge-semantic/    # Semantic analysis
│   ├── forge-bayesnet/    # Bayesian network
│   ├── forge-ml/          # ML inference
│   ├── forge-anticipation/ # Anticipation engine
│   ├── forge-immune/      # Anomaly detection
│   ├── forge-developer/   # Developer profiles
│   ├── forge-surfaces/    # UI surface system
│   ├── forge-feedback/    # Feedback tracking
│   ├── forge-agent/       # AI agent
│   └── forge-net/         # Network client
```

## Code Rules
1. **Rust 2021 edition**. No `.unwrap()` in production code. Use `Result<T, E>` + `thiserror`. `.unwrap()` only in `#[cfg(test)]`.
2. **Every public function** MUST have ≥1 unit test.
3. Run `cargo fmt` + `cargo clippy -- -D warnings` after every change. Zero warnings.
4. Only add dependencies listed in your task. Add to `[workspace.dependencies]` in root `Cargo.toml`.
5. **NEW FILES ONLY** within your batch. Never modify files owned by other agents in same batch.
6. `cargo check --workspace && cargo test --workspace` must pass.
7. **UTF-8, LF** line endings.
8. `///` for public API docs, `//` for internal comments.

## Creating a New Crate
```bash
mkdir crates/forge-YOUR-CRATE
# Add to root Cargo.toml [workspace] members
# Create crates/forge-YOUR-CRATE/Cargo.toml
# Create crates/forge-YOUR-CRATE/src/lib.rs
```

## Verification (MANDATORY before submitting)
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo check --workspace
```
ALL must exit 0.
