# Jules Session 1 — Copy-Paste Prompts (9 Agents: 2-10)

> No Agent 1 needed. Open 9 Jules sessions on `SMSLYCLOUD/forge` (master). Paste ONE prompt per session.

---

## AGENT 2 — Enhance forge-config + forge-keybindings crate

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- forge-config already exists at crates/forge-config/ with structs: Config, EditorConfig, TypographyConfig, TerminalConfig, KeybindingConfig, OnboardingConfig
- Config already has load_from_str() and load_from_file() methods
- It already uses serde, toml, anyhow
- forge-input already exists at crates/forge-input/ with Key, Modifiers, Keybinding, Command, Keymap structs and default_vscode() keymap
- The root Cargo.toml already has dirs-next, toml, serde, anyhow in [workspace.dependencies]
- DO NOT create a new forge-keybindings crate — keybindings already live in forge-input

Follow these steps IN ORDER. Do not skip steps. Do not proceed until each step compiles.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_02.md for context.

STEP 2: Open and read crates/forge-config/src/lib.rs. Note what already exists.

STEP 3: Open and read crates/forge-config/Cargo.toml. Note current dependencies.

STEP 4: Add a save_to_file() method to the existing Config impl block:
  pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
      let content = toml::to_string_pretty(self)?;
      if let Some(parent) = path.as_ref().parent() {
          std::fs::create_dir_all(parent)?;
      }
      std::fs::write(path, content)?;
      Ok(())
  }

STEP 5: Add a config_dir() function that returns the config directory path:
  pub fn config_dir() -> Option<std::path::PathBuf> {
      dirs_next::config_dir().map(|d| d.join("forge"))
  }

STEP 6: Add a load_or_default() method:
  pub fn load_or_default() -> Self {
      if let Some(dir) = Self::config_dir() {
          let path = dir.join("config.toml");
          if path.exists() {
              return Self::load_from_file(path).unwrap_or_default();
          }
      }
      Self::default()
  }

STEP 7: Make sure dirs-next is in crates/forge-config/Cargo.toml dependencies. If not, add: dirs-next = { workspace = true }

STEP 8: Run `cargo check -p forge-config`. Fix any errors.

STEP 9: Add tests to crates/forge-config/src/lib.rs in a #[cfg(test)] mod tests block:
  - test_default_config: Config::default() creates valid config with expected values
  - test_round_trip: create Config::default(), save_to_file() to a tempdir, load_from_file() back, compare fields match
  Use std::env::temp_dir() for temp paths.

STEP 10: Run `cargo test -p forge-config`. Fix failures.

STEP 11: Run `cargo clippy -p forge-config -- -D warnings`. Fix warnings.

STEP 12: Run `cargo fmt --check`. Fix formatting.

DONE. Do NOT create any new crates. Only modify crates/forge-config/.
```

---

## AGENT 3 — Enhance forge-theme + Create forge-icons

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- forge-theme already exists at crates/forge-theme/ with: Color (hex wrapper), Theme, SyntaxColors, UiColors, DiagnosticColors
- Theme already has forge_night() and forge_day() constructors with full color maps
- Color uses String internally with new(hex) constructor
- forge-theme Cargo.toml already uses serde, anyhow
- DO NOT replace or rewrite Theme/Color — they are already complete

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_03.md.

STEP 2: Open and read crates/forge-theme/src/lib.rs completely. Understand what exists.

STEP 3: Add a parse_hex_to_f32 method to the Color impl:
  pub fn to_f32_array(&self) -> [f32; 4] {
      let hex = self.0.trim_start_matches('#');
      let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
      let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
      let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
      [r, g, b, 1.0]
  }

STEP 4: Add a color() method to Theme that looks up a UI color by name:
  pub fn color(&self, key: &str) -> [f32; 4] {
      match key {
          "editor.background" => self.ui.editor_bg.to_f32_array(),
          "editor.foreground" => self.ui.foreground.to_f32_array(),
          "sideBar.background" => self.ui.sidebar_bg.to_f32_array(),
          // ... map all UiColors fields
          _ => [0.8, 0.8, 0.8, 1.0], // fallback
      }
  }

STEP 5: Run `cargo check -p forge-theme`. Fix errors.

STEP 6: Add tests: Color::new("#ff8000").unwrap().to_f32_array() approximately equals [1.0, 0.502, 0.0, 1.0]. Theme::forge_night().color("editor.background") returns non-zero values.

STEP 7: Run `cargo test -p forge-theme`. All pass.

STEP 8: Create directory crates/forge-icons/src/

STEP 9: Create crates/forge-icons/Cargo.toml:
  [package]
  name = "forge-icons"
  version = "0.1.0"
  edition = "2021"

STEP 10: Create crates/forge-icons/src/lib.rs with:
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum FileIcon { Rust, JavaScript, TypeScript, Python, Go, C, Cpp, Json, Toml, Yaml, Html, Css, Markdown, Shell, Docker, Git, Generic }

  impl FileIcon {
      pub fn from_extension(ext: &str) -> Self { match ext { "rs" => Self::Rust, "js" | "mjs" | "cjs" => Self::JavaScript, "ts" | "tsx" => Self::TypeScript, "py" => Self::Python, "go" => Self::Go, "c" | "h" => Self::C, "cpp" | "hpp" | "cc" => Self::Cpp, "json" => Self::Json, "toml" => Self::Toml, "yaml" | "yml" => Self::Yaml, "html" | "htm" => Self::Html, "css" | "scss" => Self::Css, "md" => Self::Markdown, "sh" | "bash" | "zsh" => Self::Shell, "dockerfile" => Self::Docker, "gitignore" => Self::Git, _ => Self::Generic } }
      pub fn glyph(&self) -> &'static str { match self { Self::Rust => "🦀", Self::JavaScript => "📜", Self::TypeScript => "🔷", Self::Python => "🐍", Self::Go => "🐹", Self::C | Self::Cpp => "⚙️", Self::Json => "📋", Self::Toml => "⚙️", Self::Yaml => "📄", Self::Html => "🌐", Self::Css => "🎨", Self::Markdown => "📝", Self::Shell => "🐚", Self::Docker => "🐳", Self::Git => "📂", Self::Generic => "📄" } }
  }

  Add tests: from_extension("rs") == Rust, from_extension("xyz") == Generic

STEP 11: Add "crates/forge-icons" to [workspace] members in root Cargo.toml (add it after the last entry in the members array).

STEP 12: Run `cargo check -p forge-icons`. Fix errors.

STEP 13: Run `cargo test -p forge-icons`. All pass.

STEP 14: Run `cargo clippy -- -D warnings` on both crates. Fix warnings.

STEP 15: Run `cargo fmt`.

DONE.
```

---

## AGENT 4 — Buffer Edge-Case Tests + Clipboard + Recovery

```
You are working on the Forge editor, a Rust workspace at the root of this repo.

IMPORTANT CONTEXT — READ BEFORE DOING ANYTHING:
- Buffer lives at crates/forge-core/src/buffer.rs
- Buffer API includes: new(), from_str(s), from_file(path), text(), len_lines(), len_bytes(), slice(start, end), apply(Transaction), undo(), redo(), is_dirty(), mark_clean(), save(), save_as(), selection(), set_selection(), offset_to_line_col(), line_col_to_offset()
- Buffer uses ropey::Rope internally
- Existing tests: test_buffer_creation, test_buffer_transactions, test_buffer_undo_redo, test_offset_to_line_col
- forge-input already exists at crates/forge-input/
- forge-core's lib.rs declares: mod buffer; pub mod git; mod history; pub mod layout; mod position; pub mod project; mod selection; pub mod syntax; pub mod terminal; mod transaction;
- Root Cargo.toml already has arboard = "3" and dirs-next = "2" in workspace deps

Follow these steps IN ORDER.

STEP 1: Read tasks/GLOBAL_RULES.md and tasks/session1/agent_04.md.

STEP 2: Open crates/forge-core/src/buffer.rs. Read the ENTIRE file. Note all existing methods and the existing #[cfg(test)] mod tests block.

STEP 3: ADD new tests to the EXISTING #[cfg(test)] mod tests block (do NOT create a duplicate test module). Add these tests using the exact Buffer API:

  #[test]
  fn test_empty_buffer_has_one_line() {
      let buf = Buffer::new();
      assert_eq!(buf.len_lines(), 1);
  }

  #[test]
  fn test_emoji_insertion() {
      let buf = Buffer::from_str("Hello 👋🌍");
      assert!(buf.text().contains("👋🌍"));
  }

  #[test]
  fn test_cjk_characters() {
      let buf = Buffer::from_str("你好世界");
      assert_eq!(buf.text().trim(), "你好世界");
  }

  #[test]
  fn test_large_buffer() {
      let content: String = (0..100_000).map(|i| format!("line {}\n", i)).collect();
      let buf = Buffer::from_str(&content);
      assert_eq!(buf.len_lines(), 100_001); // 100000 lines + trailing
  }

  #[test]
  fn test_buffer_dirty_tracking() {
      let mut buf = Buffer::from_str("hello");
      assert!(!buf.is_dirty());
      // Apply a transaction to make it dirty
      let tx = Transaction::new(vec![ChangeSet::new(vec![
          Change::Retain(5),
          Change::Insert(" world".to_string()),
      ])]);
      buf.apply(tx);
      assert!(buf.is_dirty());
      buf.mark_clean();
      assert!(!buf.is_dirty());
  }

STEP 4: Run `cargo test -p forge-core`. If any test fails because the API doesn't match, READ the actual method signatures and fix the test code. Do NOT modify non-test code.

STEP 5: Open crates/forge-input/src/lib.rs. Read it to see what modules exist.

STEP 6: Create crates/forge-input/src/clipboard.rs:
  use anyhow::Result;

  pub struct Clipboard {
      inner: arboard::Clipboard,
  }

  impl Clipboard {
      pub fn new() -> Result<Self> {
          Ok(Self { inner: arboard::Clipboard::new()? })
      }
      pub fn copy(&mut self, text: &str) -> Result<()> {
          self.inner.set_text(text)?;
          Ok(())
      }
      pub fn paste(&mut self) -> Result<String> {
          Ok(self.inner.get_text()?)
      }
  }

STEP 7: Open crates/forge-input/src/lib.rs and add `pub mod clipboard;` at the top.

STEP 8: Open crates/forge-input/Cargo.toml. Add these if not present:
  arboard = { workspace = true }
  anyhow = { workspace = true }

STEP 9: Run `cargo check -p forge-input`. Fix errors.

STEP 10: Create crates/forge-core/src/recovery.rs:
  use anyhow::Result;
  use std::path::{Path, PathBuf};
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};

  pub struct RecoveryManager { recovery_dir: PathBuf }

  impl RecoveryManager {
      pub fn new() -> Result<Self> {
          let dir = dirs_next::data_dir()
              .unwrap_or_else(|| PathBuf::from("."))
              .join("forge").join("recovery");
          std::fs::create_dir_all(&dir)?;
          Ok(Self { recovery_dir: dir })
      }
      pub fn with_dir(dir: PathBuf) -> Result<Self> {
          std::fs::create_dir_all(&dir)?;
          Ok(Self { recovery_dir: dir })
      }
      fn hash_path(path: &str) -> String {
          let mut h = DefaultHasher::new();
          path.hash(&mut h);
          format!("{:x}.forge-recovery", h.finish())
      }
      pub fn save(&self, file_path: &str, content: &str) -> Result<()> {
          let name = Self::hash_path(file_path);
          std::fs::write(self.recovery_dir.join(name), content)?;
          Ok(())
      }
      pub fn recover(&self, file_path: &str) -> Result<Option<String>> {
          let name = Self::hash_path(file_path);
          let p = self.recovery_dir.join(name);
          if p.exists() { Ok(Some(std::fs::read_to_string(p)?)) } else { Ok(None) }
      }
      pub fn clear(&self, file_path: &str) -> Result<()> {
          let name = Self::hash_path(file_path);
          let p = self.recovery_dir.join(name);
          if p.exists() { std::fs::remove_file(p)?; }
          Ok(())
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn test_save_and_recover() {
          let dir = std::env::temp_dir().join("forge-recovery-test");
          let _ = std::fs::remove_dir_all(&dir);
          let mgr = RecoveryManager::with_dir(dir.clone()).unwrap();
          mgr.save("test.rs", "fn main() {}").unwrap();
          let recovered = mgr.recover("test.rs").unwrap();
          assert_eq!(recovered, Some("fn main() {}".to_string()));
          mgr.clear("test.rs").unwrap();
          assert_eq!(mgr.recover("test.rs").unwrap(), None);
          let _ = std::fs::remove_dir_all(&dir);
      }
  }

STEP 11: Open crates/forge-core/src/lib.rs and add `pub mod recovery;` in the module declarations.

STEP 12: Open crates/forge-core/Cargo.toml. Make sure `dirs-next = { workspace = true }` is in [dependencies]. Add it if not.

STEP 13: Run `cargo check -p forge-core`. Fix errors.

STEP 14: Run `cargo test -p forge-core`. All tests pass.

STEP 15: Run `cargo test -p forge-input`. All tests pass.

STEP 16: Run `cargo clippy -p forge-core -p forge-input -- -D warnings`. Fix warnings.

STEP 17: Run `cargo fmt`.

DONE.
```
