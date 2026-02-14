# Forge 1.0 Master Roadmap: The Path to Parity

This document outlines the strategic plan to elevate **Forge** from a "Sub-Binary Prototype" to a **Production-Grade Competitor** against VS Code and Cursor.

**Target**: Feature parity with VS Code (Ecosystem/UI) + Feature parity with Cursor (AI).
**Executor**: Future AI Agents.

---

## 🏗️ Epic 1: The UI/UX Foundation (Parity with Sublime/Notepad++)

**Goal**: Make the editor feel "real", flexible, and accessible.

### 1.1 Flexible Docking System
- **Current State**: Hardcoded `LayoutZones` in `forge-app`.
- **Task**: Implement a recursive tiling window manager within the app.
    - [ ] Create `DockManager` in `forge-app`.
    - [ ] Support drag-and-drop of panels (Terminal, AI, Explorer) between zones.
    - [ ] Serialize layout state to `workspace.toml`.

### 1.2 Rich Text & Markdown
- **Current State**: Basic `glyphon` text rendering. No image/link support in chat.
- **Task**: Implement a rich text layout engine.
    - [ ] Upgrade `forge-renderer` to support varying font sizes/weights in the same buffer (for Markdown).
    - [ ] Implement image rendering in `ChatPanel` (using `wgpu` texture overlays).
    - [ ] Support clickable links (LSP definitions, web URLs).

### 1.3 Command Palette Polish
- **Current State**: Functional but basic string matching.
- **Task**: Implement "Quick Open" vs "Command Palette".
    - [ ] Separate `Ctrl+P` (Files) and `Ctrl+Shift+P` (Commands).
    - [ ] Integrate `forge-search` (fuzzy matching) for file results.
    - [ ] Add "Recent Files" priority.

### 1.4 Accessibility (A11y)
- **Current State**: None.
- **Task**: Integrate `accesskit`.
    - [ ] Map `Editor` buffer to an accessibility tree.
    - [ ] Ensure screen readers can read the active line and selection.

---

## 🌉 Epic 2: The VS Code Bridge (The "Antigravity" Feature)

**Goal**: Run VS Code extensions directly. This is the **most critical** adoption blocker.

### 2.1 Extension Host Architecture
- **Current State**: Custom WASM plugins only.
- **Task**: Create a Node.js-compatible extension host.
    - [ ] Integrate `deno_core` or spawn a sidecar Node.js process (`forge-node-host`).
    - [ ] Implement the VS Code Extension Protocol (RPC over IPC).
    - [ ] Create a `forge-extension-host` crate to manage these processes.

### 2.2 API Shim Layer (`vscode` namespace)
- **Current State**: None.
- **Task**: Mock the VS Code API in the host.
    - [ ] Implement `vscode.window`, `vscode.workspace`, `vscode.languages`.
    - [ ] Map `vscode.window.createTextEditor` to Forge's `Editor` via RPC.
    - [ ] Map `vscode.languages.registerCompletionItemProvider` to Forge's `LspClient`.

### 2.3 TextMate Grammar Support
- **Current State**: Tree-sitter only.
- **Task**: Support TextMate grammars for extensions that don't use Tree-sitter.
    - [ ] Integrate `syntect` or a Rust TextMate parser.
    - [ ] Allow extensions to contribute grammars via `package.json`.

---

## 🧠 Epic 3: AI Singularity (Parity with Cursor)

**Goal**: The editor *knows* your code, it doesn't just read it.

### 3.1 Local Vector Database
- **Current State**: Stubbed.
- **Task**: Integrate a real embedding store.
    - [ ] Add `sqlite-vec` or `lance` dependency to `forge-semantic`.
    - [ ] Implement background indexing of the workspace (chunking strategy).
    - [ ] Wire `Agent` to query this DB for RAG context.

### 3.2 Shadow Workspace
- **Current State**: None.
- **Task**: Create a headless editor instance for the AI.
    - [ ] Implement `ShadowBuffer` (in-memory, no render).
    - [ ] Allow the AI to apply edits to the shadow buffer, run `cargo check` / `tsc`, and verify fixes *before* showing the user.

### 3.3 Inline Diff Streaming
- **Current State**: Chat only.
- **Task**: Allow AI to write directly to the editor with a diff view.
    - [ ] Implement "Ghost Text" (gray text) for predictive edits.
    - [ ] Implement "Inline Diff" (green/red background) for AI suggestions.
    - [ ] Allow `Tab` to accept ghost text.

---

## 🛠️ Epic 4: Deep Editing (The "Vim" Factor)

**Goal**: Power-user editing capabilities.

### 4.1 Advanced Multicursor
- **Current State**: Basic multiple carets.
- **Task**: Full VS Code parity.
    - [ ] `Ctrl+D`: Select next occurrence.
    - [ ] `Alt+Click`: Add cursor.
    - [ ] Copy/Paste behavior with multiple cursors (n cursors -> n lines).

### 4.2 Large File Optimization
- **Current State**: `ropey` is good, but render loop is naive.
- **Task**: Virtualization and async loading.
    - [ ] Ensure files > 1GB load instantly (streaming read).
    - [ ] Implement "Read Only" mode for massive files to skip Tree-sitter if it chokes.

### 4.3 Search & Replace
- **Current State**: Basic in-memory search.
- **Task**: Ripgrep integration.
    - [ ] Integrate `ripgrep` binary or library for workspace-wide search.
    - [ ] Build a "Search Results" panel (virtualized list).

---

## 🐞 Epic 5: Full Debugging Suite

**Goal**: Stop using `println!`.

### 5.1 DAP UI
- **Current State**: Breakpoints toggle, backend connects.
- **Task**: Build the Debug UI panels.
    - [ ] **Variables View**: Tree view of locals/globals.
    - [ ] **Watch View**: User-defined expressions.
    - [ ] **Call Stack**: Interactive stack frames (click to jump).
    - [ ] **Debug Toolbar**: Continue, Step Over, Step Into, Step Out, Stop.

### 5.2 Hover Inspection
- **Current State**: None.
- **Task**: Show variable values on hover during debug session.
    - [ ] Query DAP for `evaluate` on hover.

---

## 📦 Epic 6: Platform & Distribution

**Goal**: Get it on people's machines.

### 6.1 Settings Sync
- **Current State**: Local `forge.toml`.
- **Task**: Cloud sync.
    - [ ] Implement GitHub Gist or custom backend sync for settings/keybindings.

### 6.2 Auto-Updater
- **Current State**: `cargo install`.
- **Task**: Binary distribution.
    - [ ] Implement `forge-updater` (check release, download, swap binary).

### 6.3 Telemetry (Opt-in)
- **Current State**: None.
- **Task**: Crash reporting.
    - [ ] Integrate `sentry` or similar for panic reporting (strictly opt-in).

---

## 📝 Execution Order for Agents

1.  **UI/UX (Epic 1)**: Must look good to be taken seriously.
2.  **Debugging (Epic 5)**: Essential for "dogfooding" (debugging Forge with Forge).
3.  **AI (Epic 3)**: The competitive differentiator.
4.  **VS Code Bridge (Epic 2)**: The ecosystem unlock (hardest/longest task).
5.  **Platform (Epic 6)**: Release prep.
