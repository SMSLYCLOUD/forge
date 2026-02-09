## AGENT 8 — Tab Manager + File I/O

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- crates/forge-app/src/editor.rs has the Editor struct with these methods:
  - Editor::new() -> Self (creates empty editor)
  - Editor::open_file(path: &str) -> anyhow::Result<Self> (loads file)
  - editor.text() -> String
  - editor.total_lines() -> usize
  - editor.cursor_line_col() -> (usize, usize)
  - editor.save() -> anyhow::Result<()>
  - editor.window_title() -> String
  - editor.insert_char(c), editor.backspace(), editor.delete()
  - editor.move_left/right/up/down/home/end()
  - editor.scroll(delta: f64)
  - editor.ensure_cursor_visible(visible_lines: usize)
- crates/forge-app/src/main.rs mod declarations: application, editor, extensions, gpu, modes, activity_bar, breadcrumb, cursor, guard, gutter, organism, rect_renderer, scrollbar, status_bar, tab_bar, ui
- main.rs is NOT async (plain fn main(), no tokio)
- forge-core already has: Buffer with save(), save_as(), path(), is_dirty(), text(), len_lines(), mark_clean()
- forge-core lib.rs modules: buffer, git, history, layout, position, project, selection, syntax, terminal, transaction

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_08.md.

STEP 2: Open and read crates/forge-app/src/editor.rs completely. Note the EXACT struct fields.

STEP 3: Create crates/forge-app/src/tab_manager.rs:
  use crate::editor::Editor;
  use anyhow::Result;
  use std::path::PathBuf;

  pub struct Tab {
      pub title: String,
      pub path: Option<PathBuf>,
      pub editor: Editor,
      pub is_modified: bool,
  }

  pub struct TabManager {
      pub tabs: Vec<Tab>,
      pub active: usize,
  }

  impl TabManager {
      pub fn new() -> Self { Self { tabs: Vec::new(), active: 0 } }

      pub fn open_file(&mut self, path: &str) -> Result<()> {
          let pb = PathBuf::from(path);
          // Check if already open
          for (i, tab) in self.tabs.iter().enumerate() {
              if tab.path.as_ref() == Some(&pb) {
                  self.active = i;
                  return Ok(());
              }
          }
          let editor = Editor::open_file(path)?;
          let title = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "untitled".to_string());
          self.tabs.push(Tab { title, path: Some(pb), editor, is_modified: false });
          self.active = self.tabs.len() - 1;
          Ok(())
      }

      pub fn close_tab(&mut self, idx: usize) {
          if idx < self.tabs.len() {
              self.tabs.remove(idx);
              if self.active >= self.tabs.len() && !self.tabs.is_empty() {
                  self.active = self.tabs.len() - 1;
              }
          }
      }

      pub fn close_current(&mut self) { let idx = self.active; self.close_tab(idx); }
      pub fn next_tab(&mut self) { if !self.tabs.is_empty() { self.active = (self.active + 1) % self.tabs.len(); } }
      pub fn prev_tab(&mut self) { if !self.tabs.is_empty() { self.active = if self.active == 0 { self.tabs.len() - 1 } else { self.active - 1 }; } }
      pub fn active_editor(&self) -> Option<&Editor> { self.tabs.get(self.active).map(|t| &t.editor) }
      pub fn active_editor_mut(&mut self) -> Option<&mut Editor> { self.tabs.get_mut(self.active).map(|t| &mut t.editor) }
      pub fn tab_count(&self) -> usize { self.tabs.len() }
  }

STEP 4: Open crates/forge-app/src/main.rs. Add `mod tab_manager;` in the UI components section.

STEP 5: Run `cargo check -p forge-app`. Fix errors. Common issues:
  - If Editor fields are private, use only public methods
  - If Editor::open_file() signature differs, match it exactly

STEP 6: Create crates/forge-core/src/file_io.rs:
  use anyhow::Result;
  use std::path::Path;

  pub struct FileIO;

  impl FileIO {
      pub fn save_atomic(path: &Path, content: &str) -> Result<()> {
          let tmp = path.with_extension("forge-tmp");
          std::fs::write(&tmp, content)?;
          std::fs::rename(&tmp, path)?;
          Ok(())
      }

      pub fn is_binary(path: &Path) -> Result<bool> {
          let data = std::fs::read(path)?;
          let check_len = data.len().min(8192);
          Ok(data[..check_len].contains(&0u8))
      }

      pub fn detect_line_ending(content: &str) -> &'static str {
          if content.contains("\r\n") { "\r\n" } else { "\n" }
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn test_atomic_save() {
          let dir = std::env::temp_dir().join("forge-io-test");
          let _ = std::fs::create_dir_all(&dir);
          let path = dir.join("test.txt");
          FileIO::save_atomic(&path, "hello world").unwrap();
          assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello world");
          let _ = std::fs::remove_dir_all(&dir);
      }
      #[test]
      fn test_line_ending_detection() {
          assert_eq!(FileIO::detect_line_ending("hello\nworld"), "\n");
          assert_eq!(FileIO::detect_line_ending("hello\r\nworld"), "\r\n");
      }
  }

STEP 7: Open crates/forge-core/src/lib.rs. Add `pub mod file_io;` in the module declarations.

STEP 8: Run `cargo check -p forge-core`. Fix errors.

STEP 9: Run `cargo test -p forge-core`. All pass.

STEP 10: Run `cargo clippy -p forge-core -p forge-app -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 9 — forge-workspace + Project Kind Detection

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- crates/forge-core/src/project.rs already has a Project struct with: root (PathBuf), files (Vec<PathBuf>), new(), scan(), visit_dir(), is_ignored()
- DO NOT delete or replace Project — ADD your ProjectKind enum alongside it
- Root Cargo.toml already has serde, toml, anyhow in workspace deps
- The workspace currently has 19 members

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_09.md.

STEP 2: Create directory crates/forge-workspace/src/

STEP 3: Create crates/forge-workspace/Cargo.toml:
  [package]
  name = "forge-workspace"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  anyhow = { workspace = true }
  serde = { workspace = true }

STEP 4: Create crates/forge-workspace/src/lib.rs:
  use anyhow::Result;
  use serde::{Deserialize, Serialize};
  use std::path::{Path, PathBuf};

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Workspace {
      pub name: String,
      pub roots: Vec<PathBuf>,
  }

  impl Workspace {
      pub fn from_dir(path: &Path) -> Result<Self> {
          let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "workspace".to_string());
          Ok(Self { name, roots: vec![path.to_path_buf()] })
      }
      pub fn add_root(&mut self, path: PathBuf) {
          if !self.roots.contains(&path) { self.roots.push(path); }
      }
      pub fn resolve_path(&self, relative: &str) -> Option<PathBuf> {
          for root in &self.roots {
              let full = root.join(relative);
              if full.exists() { return Some(full); }
          }
          None
      }
      pub fn contains(&self, path: &Path) -> bool {
          self.roots.iter().any(|r| path.starts_with(r))
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn test_from_dir() {
          let dir = std::env::temp_dir();
          let ws = Workspace::from_dir(&dir).unwrap();
          assert_eq!(ws.roots.len(), 1);
      }
      #[test]
      fn test_add_root_dedup() {
          let dir = std::env::temp_dir();
          let mut ws = Workspace::from_dir(&dir).unwrap();
          ws.add_root(dir.clone());
          assert_eq!(ws.roots.len(), 1);
      }
      #[test]
      fn test_contains() {
          let dir = std::env::temp_dir();
          let ws = Workspace::from_dir(&dir).unwrap();
          assert!(ws.contains(&dir.join("some_file.txt")));
      }
  }

STEP 5: Add "crates/forge-workspace" to the members array in root Cargo.toml.

STEP 6: Run `cargo check -p forge-workspace`. Fix errors.

STEP 7: Run `cargo test -p forge-workspace`. All pass.

STEP 8: Open crates/forge-core/src/project.rs. Read the ENTIRE existing code.

STEP 9: ADD (do NOT delete anything) to the bottom of project.rs, before any test module:
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ProjectKind { Rust, Node, Python, Go, Generic }

  pub fn detect_project_kind(root: &Path) -> ProjectKind {
      if root.join("Cargo.toml").exists() { ProjectKind::Rust }
      else if root.join("package.json").exists() { ProjectKind::Node }
      else if root.join("pyproject.toml").exists() || root.join("setup.py").exists() { ProjectKind::Python }
      else if root.join("go.mod").exists() { ProjectKind::Go }
      else { ProjectKind::Generic }
  }

STEP 10: Add to forge-core's lib.rs re-exports: pub use project::{ProjectKind, detect_project_kind};

STEP 11: Run `cargo check -p forge-core`. Fix errors.

STEP 12: Add a test at bottom of project.rs:
  #[cfg(test)]
  mod project_kind_tests {
      use super::*;
      #[test]
      fn test_detect_rust_project() {
          // The repo root has Cargo.toml
          let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
          assert_eq!(detect_project_kind(root), ProjectKind::Rust);
      }
  }

STEP 13: Run `cargo test -p forge-workspace -p forge-core`. All pass.

STEP 14: Run `cargo clippy -p forge-workspace -p forge-core -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 10 — Decoration Framework + Selection Rendering

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- crates/forge-renderer/src/lib.rs has modules: atlas, pipeline, text, theme, viewport — and re-exports GlyphAtlas, RenderPipeline, TextRenderer, Color, Theme, Viewport
- The Rect struct in crates/forge-app/src/rect_renderer.rs has EXACTLY:
    pub struct Rect { pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub color: [f32; 4] }
- Zone struct in crates/forge-app/src/ui.rs has EXACTLY:
    pub struct Zone { pub x: f32, pub y: f32, pub width: f32, pub height: f32 }
    with methods: new(x, y, width, height), contains(px, py), to_rect(color)
- main.rs mod list: application, editor, extensions, gpu, modes, activity_bar, breadcrumb, cursor, guard, gutter, organism, rect_renderer, scrollbar, status_bar, tab_bar, ui

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_10.md.

STEP 2: Open crates/forge-renderer/src/lib.rs. Read it.

STEP 3: Create crates/forge-renderer/src/decorations.rs:
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum UnderlineStyle { Solid, Wavy, Dashed, Dotted }

  #[derive(Debug, Clone)]
  pub enum Decoration {
      Underline { line: usize, start_col: usize, end_col: usize, color: [u8; 4], style: UnderlineStyle },
      LineBackground { line: usize, color: [u8; 4] },
      InlineText { line: usize, col: usize, text: String, color: [u8; 4] },
  }

  impl Decoration {
      fn line_number(&self) -> usize {
          match self { Decoration::Underline { line, .. } | Decoration::LineBackground { line, .. } | Decoration::InlineText { line, .. } => *line }
      }
  }

  pub struct DecorationLayer { decorations: Vec<Decoration> }

  impl DecorationLayer {
      pub fn new() -> Self { Self { decorations: Vec::new() } }
      pub fn add(&mut self, dec: Decoration) { self.decorations.push(dec); }
      pub fn clear(&mut self) { self.decorations.clear(); }
      pub fn get_line_decorations(&self, line: usize) -> Vec<&Decoration> { self.decorations.iter().filter(|d| d.line_number() == line).collect() }
      pub fn count(&self) -> usize { self.decorations.len() }
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn test_decoration_layer() {
          let mut layer = DecorationLayer::new();
          layer.add(Decoration::LineBackground { line: 0, color: [255, 0, 0, 128] });
          layer.add(Decoration::LineBackground { line: 1, color: [0, 255, 0, 128] });
          layer.add(Decoration::Underline { line: 0, start_col: 0, end_col: 5, color: [255, 0, 0, 255], style: UnderlineStyle::Wavy });
          assert_eq!(layer.count(), 3);
          assert_eq!(layer.get_line_decorations(0).len(), 2);
          assert_eq!(layer.get_line_decorations(1).len(), 1);
          assert_eq!(layer.get_line_decorations(99).len(), 0);
          layer.clear();
          assert_eq!(layer.count(), 0);
      }
  }

STEP 4: Open crates/forge-renderer/src/lib.rs. Add `pub mod decorations;` and add to re-exports:
  pub use decorations::{Decoration, DecorationLayer, UnderlineStyle};

STEP 5: Run `cargo check -p forge-renderer`. Fix errors.

STEP 6: Run `cargo test -p forge-renderer`. All pass.

STEP 7: Create crates/forge-app/src/selection_render.rs:
  use crate::rect_renderer::Rect;

  pub struct SelectionRenderer;

  impl SelectionRenderer {
      /// Convert selection ranges to highlight Rects.
      /// Each range: (start_line, start_col, end_line, end_col)
      pub fn render_selections(
          selections: &[(usize, usize, usize, usize)],
          scroll_top: usize,
          zone_x: f32, zone_y: f32, zone_width: f32,
          char_width: f32, line_height: f32,
      ) -> Vec<Rect> {
          let color = [0.2, 0.4, 0.8, 0.35]; // translucent blue
          let mut rects = Vec::new();
          for &(sl, sc, el, ec) in selections {
              if sl == el {
                  // Single line selection
                  let y = zone_y + ((sl as f32) - (scroll_top as f32)) * line_height;
                  let x = zone_x + (sc as f32) * char_width;
                  let w = ((ec - sc) as f32) * char_width;
                  rects.push(Rect { x, y, width: w, height: line_height, color });
              } else {
                  // Multi-line: first line from start_col to end
                  let y0 = zone_y + ((sl as f32) - (scroll_top as f32)) * line_height;
                  rects.push(Rect { x: zone_x + (sc as f32) * char_width, y: y0, width: zone_width - (sc as f32) * char_width, height: line_height, color });
                  // Middle lines: full width
                  for line in (sl + 1)..el {
                      let y = zone_y + ((line as f32) - (scroll_top as f32)) * line_height;
                      rects.push(Rect { x: zone_x, y, width: zone_width, height: line_height, color });
                  }
                  // Last line: from 0 to end_col
                  let yn = zone_y + ((el as f32) - (scroll_top as f32)) * line_height;
                  rects.push(Rect { x: zone_x, y: yn, width: (ec as f32) * char_width, height: line_height, color });
              }
          }
          rects
      }
  }

STEP 8: Add `mod selection_render;` to crates/forge-app/src/main.rs.

STEP 9: Run `cargo check -p forge-app`. Fix errors.

STEP 10: Run `cargo clippy -p forge-renderer -p forge-app -- -D warnings` and `cargo fmt`.

DONE.
```

---

# After All 9 Complete

Merge all branches. Then for Session 2, open tasks/session2/all_agents.md — split by `## Agent XX` headings, one per Jules session.
