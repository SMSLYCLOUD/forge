# Agent 04 — forge-core Buffer Tests + Clipboard + Recovery

> **Read `tasks/GLOBAL_RULES.md` first.**

## Task A: Comprehensive Buffer Edge Case Tests

Add to `crates/forge-core/src/buffer.rs` (TESTS ONLY — no logic changes):
```rust
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    // Empty buffer ops
    #[test] fn empty_buffer_line_count() { let b = Buffer::new(); assert_eq!(b.line_count(), 1); }
    #[test] fn empty_buffer_delete_noop() { let mut b = Buffer::new(); b.delete_char(); /* should not panic */ }
    // Multi-byte UTF-8
    #[test] fn emoji_insert() { let mut b = Buffer::new(); b.insert_str("👋🌍"); assert!(b.text().contains("👋")); }
    #[test] fn cjk_insert() { let mut b = Buffer::new(); b.insert_str("你好世界"); assert_eq!(b.text().len(), "你好世界".len()); }
    #[test] fn zwj_sequence() { let mut b = Buffer::new(); b.insert_str("👨‍👩‍👧"); assert!(b.text().contains("👨‍👩‍👧")); }
    // Large file
    #[test] fn large_buffer() { let text: String = (0..100_000).map(|i| format!("line {}\n", i)).collect(); let b = Buffer::from_str(&text); assert_eq!(b.line_count(), 100_001); }
    // Line endings
    #[test] fn crlf_normalization() { let b = Buffer::from_str("hello\r\nworld"); assert!(!b.text().contains("\r\n")); }
}
```

Adapt test method names/signatures to match actual Buffer API.

## Task B: Clipboard Integration

### `crates/forge-input/src/clipboard.rs`
```rust
use anyhow::Result;

pub struct Clipboard {
    board: arboard::Clipboard,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        Ok(Self { board: arboard::Clipboard::new().map_err(|e| anyhow::anyhow!("{}", e))? })
    }
    pub fn copy(&mut self, text: &str) -> Result<()> {
        self.board.set_text(text).map_err(|e| anyhow::anyhow!("{}", e))
    }
    pub fn paste(&mut self) -> Result<String> {
        self.board.get_text().map_err(|e| anyhow::anyhow!("{}", e))
    }
}
```

Add `pub mod clipboard;` to `crates/forge-input/src/lib.rs`.

## Task C: Crash Recovery

### `crates/forge-core/src/recovery.rs`
```rust
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct RecoveryManager { recovery_dir: PathBuf }

impl RecoveryManager {
    pub fn new() -> Self {
        let dir = dirs_next::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("forge").join("recovery");
        Self { recovery_dir: dir }
    }
    pub fn save_buffer(&self, file_path: &str, content: &str) -> Result<()> {
        std::fs::create_dir_all(&self.recovery_dir)?;
        let hash = Self::path_hash(file_path);
        std::fs::write(self.recovery_dir.join(hash), content)?;
        Ok(())
    }
    pub fn recover_buffer(&self, file_path: &str) -> Result<Option<String>> {
        let hash = Self::path_hash(file_path);
        let path = self.recovery_dir.join(hash);
        if path.exists() { Ok(Some(std::fs::read_to_string(path)?)) } else { Ok(None) }
    }
    pub fn clear(&self, file_path: &str) -> Result<()> {
        let path = self.recovery_dir.join(Self::path_hash(file_path));
        if path.exists() { std::fs::remove_file(path)?; }
        Ok(())
    }
    fn path_hash(path: &str) -> String { format!("{:x}", md5_simple(path)) }
}

fn md5_simple(input: &str) -> u64 {
    input.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn save_and_recover() {
        let rm = RecoveryManager { recovery_dir: std::env::temp_dir().join("forge_test_recovery") };
        rm.save_buffer("test.rs", "fn main() {}").unwrap();
        assert_eq!(rm.recover_buffer("test.rs").unwrap(), Some("fn main() {}".into()));
        rm.clear("test.rs").unwrap();
        assert_eq!(rm.recover_buffer("test.rs").unwrap(), None);
    }
}
```

Add `pub mod recovery;` to `crates/forge-core/src/lib.rs`.

**Acceptance**: `cargo test -p forge-core -p forge-input` all pass.
