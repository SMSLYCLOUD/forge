use anyhow::Result;
use std::path::PathBuf;

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
        if rm.recovery_dir.exists() { std::fs::remove_dir_all(&rm.recovery_dir).unwrap(); }
        rm.save_buffer("test.rs", "fn main() {}").unwrap();
        assert_eq!(rm.recover_buffer("test.rs").unwrap(), Some("fn main() {}".into()));
        rm.clear("test.rs").unwrap();
        assert_eq!(rm.recover_buffer("test.rs").unwrap(), None);
    }
}
