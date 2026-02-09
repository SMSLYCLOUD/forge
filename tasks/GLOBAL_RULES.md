# Forge — ABSOLUTE Rules for ALL Agents (ZOMBIE EDITION)

> **YOU HAVE NO MEMORY. YOU HAVE NO CONTEXT. YOU KNOW NOTHING.**
>
> This document is your ENTIRE universe. Read EVERY word before writing a single line of code.
> If something is not in this document or your specific task file, IT DOES NOT EXIST.

---

## 1. WHAT IS FORGE?

Forge is a **GPU-accelerated code editor** written in Rust. It aims to replace VS Code.
It renders text directly on the GPU using `wgpu` + `glyphon`. It is NOT a web app. It is NOT Electron.
It runs as a native desktop application using `winit` for windowing.

**Forge lives in a monorepo at:** `forge/` (root directory).

---

## 2. WORKSPACE STRUCTURE (EXACT — DO NOT GUESS)

```
forge/
├── Cargo.toml              ← Workspace root. ALL crates listed here.
├── Cargo.lock              ← DO NOT MODIFY THIS FILE DIRECTLY
├── crates/                 ← ALL crate directories live here
│   ├── forge-core/         ← Rope buffer, transactions, undo/redo history
│   ├── forge-renderer/     ← wgpu GPU pipeline, rect renderer, text atlas
│   ├── forge-window/       ← winit event loop, windowing
│   ├── forge-app/          ← Main application: application.rs (THE BINARY)
│   ├── forge-config/       ← TOML configuration
│   ├── forge-theme/        ← Color themes
│   ├── forge-input/        ← Keyboard/mouse input handling
│   ├── forge-surfaces/     ← UI surface manager (zones, layout)
│   ├── forge-confidence/   ← Code confidence engine (SQLite)
│   ├── forge-propagation/  ← Change propagation engine
│   ├── forge-semantic/     ← Semantic analysis
│   ├── forge-bayesnet/     ← Bayesian network
│   ├── forge-ml/           ← ML inference
│   ├── forge-anticipation/ ← Anticipation engine
│   ├── forge-immune/       ← Anomaly detection
│   ├── forge-developer/    ← Developer profiles
│   ├── forge-feedback/     ← Feedback tracking
│   ├── forge-agent/        ← AI agent (chat, inline completions)
│   └── forge-net/          ← Network/HTTP client
├── config/                 ← Runtime config files
├── tasks/                  ← Agent task files (this directory)
└── target/                 ← Build output (DO NOT COMMIT)
```

---

## 3. THE ROOT Cargo.toml (EXACT CURRENT STATE)

When you create a NEW crate, you MUST:
1. Add its path to `[workspace] members` in the ROOT `Cargo.toml`
2. Add any new dependencies to `[workspace.dependencies]` (NOT to the crate's own Cargo.toml)
3. Reference workspace deps in your crate's Cargo.toml like: `anyhow = { workspace = true }`

**Current workspace members** (as of this writing — verify by reading `Cargo.toml` before you start):
```toml
[workspace]
members = [
    "crates/forge-core",
    "crates/forge-renderer",
    "crates/forge-window",
    "crates/forge-app",
    "crates/forge-config",
    "crates/forge-theme",
    "crates/forge-input",
    "crates/forge-confidence",
    "crates/forge-propagation",
    "crates/forge-semantic",
    "crates/forge-bayesnet",
    "crates/forge-ml",
    "crates/forge-anticipation",
    "crates/forge-immune",
    "crates/forge-developer",
    "crates/forge-surfaces",
    "crates/forge-feedback",
    "crates/forge-agent",
    "crates/forge-net",
]
resolver = "2"
```

**Current workspace dependencies** (USE THESE — do NOT invent new versions):
```toml
[workspace.dependencies]
ropey = "1"
smallvec = "1"
unicode-segmentation = "1"
encoding_rs = "0.8"
wgpu = "23"
cosmic-text = "0.12"
glyphon = "0.7"
winit = "0.30"
pollster = "0.4"
arboard = "3"
toml = "0.8"
serde = { version = "1", features = ["derive"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
proptest = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
petgraph = "0.6"
uuid = { version = "1", features = ["v4", "serde"] }
serde_json = "1"
git2 = "0.20"
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls", "gzip", "brotli"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
async-trait = "0.1"
dirs-next = "2"
rand = "0.8"
bytemuck = { version = "1", features = ["derive"] }
```

If your task requires a dependency NOT in this list, you MUST:
1. Add it to `[workspace.dependencies]` in the ROOT Cargo.toml
2. Reference it in your crate's Cargo.toml as `dep_name = { workspace = true }`
3. Add a comment explaining WHY it's needed

---

## 4. CODE RULES (MANDATORY — ZERO EXCEPTIONS)

### 4.1 Rust Edition & Error Handling
- Edition: **2021** (already set in workspace)
- **NEVER** use `.unwrap()` in production code. Use `Result<T, E>` + `thiserror` for typed errors + `anyhow` for ad-hoc errors.
- `.unwrap()` is ONLY allowed inside `#[cfg(test)]` blocks.
- `.expect("reason")` is allowed ONLY if the panic is truly unreachable and the message explains why.

### 4.2 Testing
- **Every public function** MUST have ≥1 unit test.
- Tests go in the same file: `#[cfg(test)] mod tests { ... }`
- Use `assert!`, `assert_eq!`, `assert_ne!`. Never `println!` assertions.
- Use `proptest` for property-based testing where applicable.

### 4.3 Formatting & Linting
- Run `cargo fmt` after EVERY change. Zero formatting violations.
- Run `cargo clippy -- -D warnings` after EVERY change. Zero warnings.
- If clippy gives a false positive, add `#[allow(clippy::rule_name)]` with a `// Reason:` comment.

### 4.4 File Ownership (CRITICAL)
- Within a single session/batch, each agent owns SPECIFIC files listed in their task.
- **NEVER modify files owned by another agent in the same session.**
- `application.rs` is ONLY modified by Session 5, Agents 07-10. ALL other agents must NOT touch it.
- If your crate needs to expose functionality to `application.rs`, create a public API in your `lib.rs`.

### 4.5 File Encoding
- UTF-8 encoding.
- LF line endings (not CRLF).
- No BOM.

### 4.6 Documentation
- `///` doc comments for ALL public types, traits, and functions.
- `//` inline comments for complex logic.
- Every struct must have a doc comment explaining its purpose.

---

## 5. CREATING A NEW CRATE (Step-by-Step)

If your task requires creating a new crate (e.g., `forge-terminal`):

### Step 1: Create the directory structure
```bash
mkdir -p crates/forge-terminal/src
```

### Step 2: Create `crates/forge-terminal/Cargo.toml`
```toml
[package]
name = "forge-terminal"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
# Add ONLY deps listed in your task. Reference workspace deps.

[dev-dependencies]
proptest = { workspace = true }
```

### Step 3: Create `crates/forge-terminal/src/lib.rs`
```rust
//! Forge Terminal — embedded terminal emulator with PTY support.

pub mod pty;
pub mod ansi;
// ... other modules from your task
```

### Step 4: Add to ROOT Cargo.toml
Add `"crates/forge-terminal"` to the `[workspace] members` array.

### Step 5: Verify
```bash
cargo check --workspace
```

---

## 6. VERIFICATION (MANDATORY — RUN ALL FOUR BEFORE SUBMITTING)

```bash
cargo fmt --check        # Must exit 0
cargo clippy -- -D warnings  # Must exit 0
cargo test --workspace   # Must exit 0
cargo check --workspace  # Must exit 0
```

**ALL FOUR must exit 0.** If ANY fails, fix it. Do not submit with failures.

---

## 7. COMMON MISTAKES (DO NOT MAKE THESE)

| Mistake | Why It's Wrong | Correct Approach |
|---------|---------------|-----------------|
| Adding deps to crate Cargo.toml directly | Breaks workspace consistency | Add to root `[workspace.dependencies]`, reference as `{ workspace = true }` |
| Using `.unwrap()` | Panics in production | Use `?` operator or `.map_err()` |
| Modifying `application.rs` | Only Session 5 agents touch this | Expose public API from your crate's `lib.rs` |
| Using `println!` for logging | Not structured | Use `tracing::info!()`, `tracing::debug!()`, etc. |
| Forgetting to add module to `lib.rs` | Module won't be compiled | Add `pub mod your_module;` to `lib.rs` |
| Not adding crate to workspace members | Crate won't be found | Add to root `Cargo.toml` `[workspace] members` |
| Using `String` where `&str` would work | Unnecessary allocations | Prefer `&str` for borrowed data |
| Hardcoding paths like `C:\` or `/home/` | Not cross-platform | Use `dirs_next::config_dir()` or `std::env::current_dir()` |

---

## 8. WHAT SUCCESS LOOKS LIKE

When you're done:
1. All files listed in your task exist at the correct paths.
2. All public functions have doc comments and tests.
3. `cargo fmt --check && cargo clippy -- -D warnings && cargo test --workspace && cargo check --workspace` all exit 0.
4. You have NOT modified any file NOT listed in your task.
5. Your code compiles and integrates cleanly with the existing workspace.
