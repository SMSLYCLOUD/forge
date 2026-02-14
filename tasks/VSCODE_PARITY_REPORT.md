# Forge vs. VS Code: Gap Analysis & Parity Report

This document compares the current "Production Ready" state of Forge (v0.1-Alpha) against the full feature set of Visual Studio Code. It serves as a strategic guide for identifying critical gaps, architectural divergences, and future roadmap priorities.

## 1. Executive Summary

| Component | Forge Status | VS Code Equivalent | Gap Severity | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **Workbench** | 🟡 Partial | Electron/HTML Grid | High | Forge uses fixed `LayoutZones` vs. VS Code's flexible grid/docking. |
| **Editor Core** | 🟢 Good | Monaco (TextMate) | Low | Forge uses Tree-sitter + Rope (faster, but less grammar ecosystem). |
| **Extensions** | 🔴 Nascent | Node.js Host | Critical | Forge uses WASM. API surface is <1% of VS Code API. |
| **LSP/IntelliSense** | 🟡 Basic | Full LSP Client | Medium | Basic completion/diagnostics working. Missing complex UI (Code Lens, Inlay). |
| **Debugging** | 🟡 Stubbed | DAP + UI | Medium | Basic breakpoints/stack. Missing visualizers, inline values, REPL. |
| **Terminal** | 🟢 Good | xterm.js | Low | Native Pty integration is performant. Missing complex shell integration. |
| **SCM (Git)** | 🟡 Basic | Git Integration | Medium | Basic status/diff. Missing graph, complex merge/rebase UI. |
| **Remote** | 🔴 None | Remote Tunnels | High | No SSH/WSL/Container support. |

---

## 2. Detailed Component Analysis

### 2.1 Workbench & Shell

**VS Code:**
- **Layout:** Fully flexible grid. Drag-and-drop any view to any location. Sidebar/Panel can move left/right/bottom.
- **Theming:** JSON-based themes controlling thousands of color tokens. Icon themes.
- **Settings:** Cascading JSON (User > Workspace > Folder). GUI editor with search.
- **Keybindings:** JSON dispatch with `when` clause contexts (e.g., `editorTextFocus && !suggestWidgetVisible`).

**Forge:**
- **Layout:** Hardcoded `LayoutZones` (ActivityBar, Sidebar, Editor, Panel). No drag-and-drop.
- **Theming:** Implements VS Code color keys (`forge-theme`), but static scope.
- **Settings:** Struct-based config (`forge-config`). Basic UI.
- **Keybindings:** Basic match in `handle_input`. No complex context engine (`when` clauses).

**Gap:**
- Lack of a flexible layout engine limits "Pro" usage (e.g., side-by-side terminals + debug console).
- Keybinding context system is required for rich extension interactions.

### 2.2 Editor Core

**VS Code (Monaco):**
- **Text Model:** Piece Tree (C++ backend).
- **Syntax:** TextMate (Regex based). Huge ecosystem.
- **Features:** Minimap (render based), Sticky Scroll, Breadcrumbs, Inlay Hints, Code Lens, Folding, Bracket Pair Colorization.

**Forge:**
- **Text Model:** Rope (`ropey`). Extremely fast.
- **Syntax:** Tree-sitter (Parsing based). More accurate, harder to write grammars.
- **Features:**
    - ✅ Multicursor
    - ✅ Gutter/Line Numbers
    - ✅ Syntax Highlighting
    - ✅ Scrollbar
    - ✅ Breadcrumbs (Basic)
    - ❌ Minimap (Stub/Basic)
    - ❌ Sticky Scroll
    - ❌ Code Lens / Inlay Hints

**Gap:**
- Forge wins on performance potential (Tree-sitter > TextMate).
- Missing "nice-to-have" visual aids (Sticky Scroll, Code Lens) that developers rely on.

### 2.3 Extension Ecosystem

**VS Code:**
- **Runtime:** Node.js (separate process). Access to FS, Network, NPM ecosystem.
- **UI:** Webviews (HTML/CSS in iframe). Custom TreeViews, Webview Panels.
- **API:** Massive `vscode.d.ts` covering almost every aspect of the UI.

**Forge:**
- **Runtime:** WASM (`wasmtime`). Sandboxed, language-agnostic, but restricted.
- **UI:** Limited `Host API` for registering commands/panels. No HTML/Webview support.
- **API:** Minimal (Log, Notification, Command).

**Gap:**
- **The Chasm:** Existing VS Code extensions **cannot** run on Forge. This is a strategic choice (performance/security vs. ecosystem).
- **Webviews:** Lack of HTML rendering means no Markdown Preview, no Git Graph, no complex UI extensions.

### 2.4 Intelligence (LSP & AI)

**VS Code:**
- **LSP:** Mature client. Semantic tokens, Call Hierarchy, Type Hierarchy, Rename, Refactor actions.
- **AI:** Copilot (separate extension). Inline completions (Ghost text), Chat panel, Inline Chat.

**Forge:**
- **LSP:** Basic client. GoToDef, Diagnostics, Hover. Missing sophisticated refactoring UI.
- **AI:** Native `forge-agent`. integrated `/explain`.
    - ✅ Integrated at binary level (faster potential).
    - ❌ Missing "Ghost Text" (inline suggestion stream).

### 2.5 Debugging

**VS Code:**
- **DAP:** Generic adapter support.
- **UI:** Watch, Call Stack, Variables, Breakpoints (Log/Conditional), Debug Console (REPL), Inline Values.

**Forge:**
- **DAP:** Basic client (`forge-debug`).
- **UI:** Sidebar text view (Stack/Vars/Breakpoints).
- **Missing:**
    - Inline variable values in editor.
    - Hover to inspect.
    - Debug Console/REPL.
    - Conditional Breakpoints.

### 2.6 Source Control

**VS Code:**
- Integrated SCM provider API.
- Inline Diff Editor.
- Merge Conflict "Accept Current/Incoming" Code Lens.

**Forge:**
- Basic `GitIntegration`.
- `GitPanel` lists changed files.
- Missing rich diff editor and conflict resolution UI.

## 3. Strategic Recommendations

1.  **Prioritize "Context Key" System:** To reach parity in UX feel, Forge needs a centralized event/context system (like VS Code's `when` clauses) to manage keybindings and menu visibility dynamically.
2.  **Webview Strategy:** Decide on a UI extension model. Embedding a browser engine (Servo/Wry) is heavy but enables Markdown/Preview. Sticking to native widgets is fast but limits extension UI creativity.
3.  **Extension Bridge:** Consider a "Compat Layer" that runs VS Code extensions in a Node sidecar and proxies API calls, if ecosystem compatibility is a goal. If not, double down on the WASM API.
4.  **Tree-sitter vs. TextMate:** Stick with Tree-sitter. It is the future. TextMate is legacy tech that VS Code is slowly moving away from (Semantic Tokens).

## 4. Conclusion

Forge is a **Sub-Binary / High-Performance** alternative. It achieves ~20% of VS Code's surface area (the critical editing/navigation loop) with potentially 10x the performance. It currently lacks the "Platform" depth (Extensions, flexible UI, Remote) that makes VS Code the standard.

**Next Phase:** Deepen the Editor Core (Sticky Scroll, Code Lens) and expand the Extension Host capabilities to allow community contribution to fill the gaps.
