use anyhow::Result;
use std::path::Path;

pub struct FileIO;

impl FileIO {
    /// Atomic save: write to temp, then rename.
    pub fn save_atomic(path: &Path, content: &str) -> Result<()> {
        let tmp = path.with_extension("forge-tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Detect if file is binary (contains null bytes in first 8KB).
    pub fn is_binary(path: &Path) -> Result<bool> {
        let data = std::fs::read(path)?;
        let check_len = data.len().min(8192);
        Ok(data[..check_len].contains(&0))
    }

    /// Detect line ending style from content.
    pub fn detect_line_ending(content: &str) -> &'static str {
        if content.contains("\r\n") { "\r\n" } else { "\n" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn atomic_save() {
        let dir = std::env::temp_dir().join("forge_io_test");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("test.txt");
        FileIO::save_atomic(&path, "hello").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
        std::fs::remove_dir_all(&dir).ok();
    }
    #[test] fn detect_lf() { assert_eq!(FileIO::detect_line_ending("a\nb\nc"), "\n"); }
    #[test] fn detect_crlf() { assert_eq!(FileIO::detect_line_ending("a\r\nb\r\nc"), "\r\n"); }
}
