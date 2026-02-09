## AGENT 5 — forge-syntax: Tree-sitter Parser + Language Detection

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- forge-syntax does NOT exist yet. You are creating it from scratch.
- forge-core already has a `syntax` module (crates/forge-core/src/syntax.rs) — do NOT touch it. Your new crate is separate.
- Root Cargo.toml already has these workspace deps: ropey, wgpu, winit, serde, toml, anyhow, thiserror, etc. Tree-sitter deps are NOT yet added.
- The workspace currently has 19 members listed in root Cargo.toml.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_05.md.

STEP 2: Run `cargo search tree-sitter --limit 5` to find the exact crate names and latest versions on crates.io. Also run:
  cargo search tree-sitter-rust --limit 3
  cargo search tree-sitter-javascript --limit 3
  cargo search tree-sitter-python --limit 3
  cargo search tree-sitter-json --limit 3
  cargo search tree-sitter-toml --limit 3

STEP 3: Based on step 2 results, add tree-sitter workspace dependencies to root Cargo.toml under [workspace.dependencies]. Add them AFTER the existing "# AI Agent" section. Example (adjust versions based on step 2):
  # Syntax Highlighting
  tree-sitter = "0.24"
  tree-sitter-rust = "0.23"
  tree-sitter-javascript = "0.23"
  tree-sitter-python = "0.23"
  tree-sitter-json = "0.24"

STEP 4: Add "crates/forge-syntax" to the members array in root Cargo.toml [workspace] section.

STEP 5: Create crates/forge-syntax/Cargo.toml:
  [package]
  name = "forge-syntax"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  tree-sitter = { workspace = true }
  tree-sitter-rust = { workspace = true }
  tree-sitter-javascript = { workspace = true }
  tree-sitter-python = { workspace = true }
  tree-sitter-json = { workspace = true }
  anyhow = { workspace = true }
  (Add any other tree-sitter grammars you found in step 2)

STEP 6: Create crates/forge-syntax/src/language.rs:
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Language { Rust, JavaScript, TypeScript, Python, Go, C, Cpp, Json, Toml, Yaml, Html, Css, Markdown, Shell, Unknown }

  impl Language {
      pub fn from_extension(ext: &str) -> Self {
          match ext {
              "rs" => Self::Rust, "js" | "mjs" | "cjs" | "jsx" => Self::JavaScript,
              "ts" | "tsx" => Self::TypeScript, "py" => Self::Python, "go" => Self::Go,
              "c" | "h" => Self::C, "cpp" | "cc" | "hpp" => Self::Cpp,
              "json" => Self::Json, "toml" => Self::Toml, "yaml" | "yml" => Self::Yaml,
              "html" | "htm" => Self::Html, "css" | "scss" => Self::Css,
              "md" | "markdown" => Self::Markdown, "sh" | "bash" | "zsh" => Self::Shell,
              _ => Self::Unknown,
          }
      }
      pub fn from_path(path: &std::path::Path) -> Self {
          path.extension().and_then(|e| e.to_str()).map(Self::from_extension).unwrap_or(Self::Unknown)
      }
      pub fn tree_sitter_language(&self) -> Option<tree_sitter::Language> {
          // CRITICAL: Check the actual crate API. Try LANGUAGE.into() first, fall back to language() function.
          match self {
              Self::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
              Self::JavaScript | Self::TypeScript => Some(tree_sitter_javascript::LANGUAGE.into()),
              Self::Python => Some(tree_sitter_python::LANGUAGE.into()),
              Self::Json => Some(tree_sitter_json::LANGUAGE.into()),
              _ => None,
          }
      }
  }

STEP 7: Run `cargo check -p forge-syntax`. THIS WILL LIKELY FAIL because tree-sitter grammar APIs vary. Read the error messages carefully:
  - If error says "no item named LANGUAGE" then try `language()` function instead: tree_sitter_rust::language()
  - If error says "expected Language, found LanguageFn" then add .into()
  - If a crate version doesn't exist, go back and fix the version in root Cargo.toml
  Keep fixing until `cargo check -p forge-syntax` passes.

STEP 8: Create crates/forge-syntax/src/parser.rs:
  use anyhow::{Result, anyhow};

  pub struct SyntaxParser {
      parser: tree_sitter::Parser,
  }

  impl SyntaxParser {
      pub fn new(lang: tree_sitter::Language) -> Result<Self> {
          let mut parser = tree_sitter::Parser::new();
          parser.set_language(&lang).map_err(|e| anyhow!("Failed to set language: {}", e))?;
          Ok(Self { parser })
      }
      pub fn parse(&mut self, text: &str) -> Result<tree_sitter::Tree> {
          self.parser.parse(text, None).ok_or_else(|| anyhow!("Parse failed"))
      }
      pub fn reparse(&mut self, text: &str, old_tree: &tree_sitter::Tree) -> Result<tree_sitter::Tree> {
          self.parser.parse(text, Some(old_tree)).ok_or_else(|| anyhow!("Reparse failed"))
      }
  }

STEP 9: Run `cargo check -p forge-syntax`. Fix errors. Common issue: set_language might take Language not &Language — check the API.

STEP 10: Create crates/forge-syntax/src/lib.rs:
  pub mod language;
  pub mod parser;
  pub use language::Language;
  pub use parser::SyntaxParser;

STEP 11: Run `cargo check -p forge-syntax`. Fix errors.

STEP 12: Add tests to crates/forge-syntax/src/lib.rs:
  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn test_language_detection() {
          assert_eq!(Language::from_extension("rs"), Language::Rust);
          assert_eq!(Language::from_extension("py"), Language::Python);
          assert_eq!(Language::from_extension("xyz"), Language::Unknown);
      }
      #[test]
      fn test_parse_rust() {
          let lang = Language::Rust.tree_sitter_language().unwrap();
          let mut parser = SyntaxParser::new(lang).unwrap();
          let tree = parser.parse("fn main() {}").unwrap();
          assert_eq!(tree.root_node().kind(), "source_file");
      }
  }

STEP 13: Run `cargo test -p forge-syntax`. All pass.

STEP 14: Run `cargo clippy -p forge-syntax -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 6 — Syntax Highlighter + Token Colors

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- crates/forge-syntax/ may or may NOT exist yet (Agent 5 creates it in parallel).
- If forge-syntax does NOT exist, you MUST create it first by following tasks/session1/agent_05.md steps.
- If forge-syntax DOES exist, just add your highlighter files to it.
- DO NOT touch any other crates.

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_06.md.

STEP 2: Check if crates/forge-syntax/src/lib.rs exists. Run: ls crates/forge-syntax/src/ 2>/dev/null || echo "NOT FOUND"

STEP 3: If NOT FOUND: Read tasks/session1/agent_05.md and execute ALL its steps first to create forge-syntax. Run `cargo check -p forge-syntax` to confirm. Then continue with step 4.

STEP 4: Create crates/forge-syntax/src/highlighter.rs:
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum TokenType { Keyword, Function, Type, StringLiteral, Number, Comment, Operator, Punctuation, Variable, Constant, Namespace, Property, Parameter, Macro, Attribute, Label, Builtin, Plain }

  #[derive(Debug, Clone)]
  pub struct HighlightSpan { pub start_byte: usize, pub end_byte: usize, pub token_type: TokenType }

  pub struct Highlighter;

  impl Highlighter {
      pub fn highlight(tree: &tree_sitter::Tree, source: &[u8]) -> Vec<HighlightSpan> {
          let mut spans = Vec::new();
          Self::walk_node(tree.root_node(), source, &mut spans);
          spans
      }

      fn walk_node(node: tree_sitter::Node, source: &[u8], spans: &mut Vec<HighlightSpan>) {
          if node.child_count() == 0 {
              let kind = node.kind();
              let token_type = Self::classify(kind);
              if token_type != TokenType::Plain {
                  spans.push(HighlightSpan {
                      start_byte: node.start_byte(),
                      end_byte: node.end_byte(),
                      token_type,
                  });
              }
          } else {
              let mut cursor = node.walk();
              for child in node.children(&mut cursor) {
                  Self::walk_node(child, source, spans);
              }
          }
      }

      fn classify(kind: &str) -> TokenType {
          match kind {
              "line_comment" | "block_comment" | "comment" => TokenType::Comment,
              "string_literal" | "string" | "raw_string_literal" | "string_content" | "char_literal" => TokenType::StringLiteral,
              "integer_literal" | "float_literal" | "number" => TokenType::Number,
              "fn" | "let" | "mut" | "pub" | "use" | "mod" | "struct" | "enum" | "impl" |
              "if" | "else" | "match" | "for" | "while" | "loop" | "return" | "break" |
              "continue" | "const" | "static" | "trait" | "type" | "where" | "async" |
              "await" | "move" | "ref" | "self" | "super" | "crate" | "as" | "in" |
              "true" | "false" | "def" | "class" | "import" | "from" | "function" |
              "var" | "export" | "default" => TokenType::Keyword,
              "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "::" | ":" | "." | "->" | "=>" => TokenType::Punctuation,
              "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" |
              "&&" | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" => TokenType::Operator,
              _ => TokenType::Plain,
          }
      }
  }

STEP 5: Run `cargo check -p forge-syntax`. Fix errors.

STEP 6: Create crates/forge-syntax/src/colors.rs:
  use crate::highlighter::TokenType;

  pub fn default_color(token: TokenType) -> [u8; 3] {
      match token {
          TokenType::Keyword => [255, 121, 198],
          TokenType::Function => [80, 250, 123],
          TokenType::Type => [139, 233, 253],
          TokenType::StringLiteral => [241, 250, 140],
          TokenType::Number => [189, 147, 249],
          TokenType::Comment => [98, 114, 164],
          TokenType::Operator => [255, 184, 108],
          TokenType::Punctuation => [248, 248, 242],
          TokenType::Macro => [187, 154, 247],
          TokenType::Attribute => [139, 233, 253],
          _ => [248, 248, 242],
      }
  }

STEP 7: Open crates/forge-syntax/src/lib.rs. Add these lines (keeping whatever already exists):
  pub mod highlighter;
  pub mod colors;
  pub use highlighter::{Highlighter, HighlightSpan, TokenType};
  pub use colors::default_color;

STEP 8: Run `cargo check -p forge-syntax`. Fix errors.

STEP 9: Add test to lib.rs tests block (or create one):
  #[test]
  fn test_highlight_rust_code() {
      let lang = Language::Rust.tree_sitter_language().unwrap();
      let mut parser = SyntaxParser::new(lang).unwrap();
      let code = "fn main() { let x = 42; }";
      let tree = parser.parse(code).unwrap();
      let spans = Highlighter::highlight(&tree, code.as_bytes());
      assert!(spans.iter().any(|s| s.token_type == TokenType::Keyword));
      assert!(spans.iter().any(|s| s.token_type == TokenType::Number));
  }

STEP 10: Run `cargo test -p forge-syntax`. All pass.

STEP 11: Run `cargo clippy -p forge-syntax -- -D warnings` and `cargo fmt`.

DONE.
```

---

## AGENT 7 — Real File Tree + File Tree UI

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- crates/forge-surfaces/src/file_explorer.rs already exists with IntelligentFileExplorer that implements SurfaceIntelligence trait
- crates/forge-surfaces/src/lib.rs exports IntelligentFileExplorer and protocol types
- DO NOT delete the existing IntelligentFileExplorer — ADD your FileNode alongside it
- crates/forge-app/src/main.rs has these mod declarations: application, editor, extensions, gpu, modes, activity_bar, breadcrumb, cursor, guard, gutter, organism, rect_renderer, scrollbar, status_bar, tab_bar, ui
- The Rect struct in rect_renderer.rs has EXACTLY these fields: pub x: f32, pub y: f32, pub width: f32, pub height: f32, pub color: [f32; 4]
- main.rs uses NO async runtime (no tokio::main, just plain fn main())

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_07.md.

STEP 2: Open and read crates/forge-surfaces/src/file_explorer.rs completely.

STEP 3: Open and read crates/forge-surfaces/src/lib.rs completely.

STEP 4: ADD these structs to crates/forge-surfaces/src/file_explorer.rs (AFTER the existing code, do NOT delete anything):

  use std::path::{Path, PathBuf};

  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum NodeKind { File, Directory }

  #[derive(Debug, Clone)]
  pub struct FileNode {
      pub name: String,
      pub path: PathBuf,
      pub kind: NodeKind,
      pub children: Vec<FileNode>,
      pub expanded: bool,
      pub depth: usize,
  }

  impl FileNode {
      pub fn build_tree(root: &Path, max_depth: usize) -> anyhow::Result<Self> {
          Self::build_recursive(root, 0, max_depth)
      }

      fn build_recursive(path: &Path, depth: usize, max_depth: usize) -> anyhow::Result<Self> {
          let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string_lossy().to_string());
          let mut node = FileNode { name, path: path.to_path_buf(), kind: NodeKind::Directory, children: Vec::new(), expanded: depth == 0, depth };

          if depth >= max_depth || !path.is_dir() {
              if !path.is_dir() { node.kind = NodeKind::File; }
              return Ok(node);
          }

          let mut entries: Vec<_> = std::fs::read_dir(path)?
              .filter_map(|e| e.ok())
              .filter(|e| {
                  let name = e.file_name().to_string_lossy().to_string();
                  !name.starts_with('.') && name != "target" && name != "node_modules"
              })
              .collect();
          entries.sort_by(|a, b| {
              let a_dir = a.path().is_dir();
              let b_dir = b.path().is_dir();
              b_dir.cmp(&a_dir).then_with(|| a.file_name().to_ascii_lowercase().cmp(&b.file_name().to_ascii_lowercase()))
          });

          for entry in entries {
              let child = Self::build_recursive(&entry.path(), depth + 1, max_depth)?;
              node.children.push(child);
          }
          Ok(node)
      }

      pub fn toggle(&mut self, target: &Path) -> bool {
          if self.path == target && self.kind == NodeKind::Directory {
              self.expanded = !self.expanded;
              return true;
          }
          for child in &mut self.children {
              if child.toggle(target) { return true; }
          }
          false
      }

      pub fn flatten_visible(&self) -> Vec<&FileNode> {
          let mut result = vec![self];
          if self.expanded {
              for child in &self.children {
                  result.extend(child.flatten_visible());
              }
          }
          result
      }
  }

STEP 5: Open crates/forge-surfaces/src/lib.rs. Add to the re-exports:
  pub use file_explorer::{FileNode, NodeKind};

STEP 6: Make sure crates/forge-surfaces/Cargo.toml has `anyhow = { workspace = true }` in [dependencies].

STEP 7: Run `cargo check -p forge-surfaces`. Fix errors.

STEP 8: Add tests at the bottom of file_explorer.rs:
  #[cfg(test)]
  mod file_tree_tests {
      use super::*;
      use std::fs;

      #[test]
      fn test_build_tree() {
          let dir = std::env::temp_dir().join("forge-tree-test");
          let _ = fs::remove_dir_all(&dir);
          fs::create_dir_all(dir.join("src")).unwrap();
          fs::write(dir.join("Cargo.toml"), "").unwrap();
          fs::write(dir.join("src/main.rs"), "").unwrap();
          let tree = FileNode::build_tree(&dir, 3).unwrap();
          assert_eq!(tree.kind, NodeKind::Directory);
          let flat = tree.flatten_visible();
          assert!(flat.len() >= 3); // root + src + files
          let _ = fs::remove_dir_all(&dir);
      }

      #[test]
      fn test_toggle() {
          let dir = std::env::temp_dir().join("forge-toggle-test");
          let _ = fs::remove_dir_all(&dir);
          fs::create_dir_all(dir.join("sub")).unwrap();
          fs::write(dir.join("sub/file.txt"), "").unwrap();
          let mut tree = FileNode::build_tree(&dir, 3).unwrap();
          let sub_path = dir.join("sub");
          let before = tree.flatten_visible().len();
          tree.toggle(&sub_path);
          let after = tree.flatten_visible().len();
          assert_ne!(before, after);
          let _ = fs::remove_dir_all(&dir);
      }
  }

STEP 9: Run `cargo test -p forge-surfaces`. All pass.

STEP 10: Open crates/forge-app/src/main.rs. Note the existing mod declarations.

STEP 11: Create crates/forge-app/src/file_tree_ui.rs:
  use crate::rect_renderer::Rect;

  pub struct DisplayNode {
      pub label: String,
      pub depth: usize,
      pub is_dir: bool,
      pub expanded: bool,
  }

  pub struct FileTreeUi {
      pub scroll_offset: f32,
      pub selected_index: Option<usize>,
      pub hovered_index: Option<usize>,
  }

  impl FileTreeUi {
      pub fn new() -> Self {
          Self { scroll_offset: 0.0, selected_index: None, hovered_index: None }
      }

      pub fn render_rects(&self, nodes: &[DisplayNode], zone_x: f32, zone_y: f32, zone_width: f32, line_height: f32) -> Vec<Rect> {
          let mut rects = Vec::new();
          for (i, _node) in nodes.iter().enumerate() {
              let y = zone_y + (i as f32 * line_height) - self.scroll_offset;
              if let Some(sel) = self.selected_index {
                  if sel == i {
                      rects.push(Rect { x: zone_x, y, width: zone_width, height: line_height, color: [0.2, 0.4, 0.8, 0.3] });
                  }
              }
              if let Some(hov) = self.hovered_index {
                  if hov == i {
                      rects.push(Rect { x: zone_x, y, width: zone_width, height: line_height, color: [1.0, 1.0, 1.0, 0.05] });
                  }
              }
          }
          rects
      }
  }

STEP 12: Open crates/forge-app/src/main.rs. Add `mod file_tree_ui;` in the "// UI components" section (after `mod ui;`).

STEP 13: Run `cargo check -p forge-app`. Fix errors — especially check that the Rect import path is correct.

STEP 14: Run `cargo clippy -p forge-surfaces -p forge-app -- -D warnings` and `cargo fmt`.

DONE.
```
