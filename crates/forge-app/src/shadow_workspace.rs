use anyhow::{anyhow, Result};
use forge_core::{Buffer, Transaction};
use std::collections::HashMap;
use std::path::PathBuf;

/// A headless workspace for AI verification.
/// It maintains a set of buffers that mirror the user's workspace but can be
/// mutated independently to verify AI suggestions before applying them.
pub struct ShadowWorkspace {
    pub buffers: HashMap<PathBuf, Buffer>,
}

impl ShadowWorkspace {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    /// Open a file into the shadow workspace (or update if exists)
    pub fn open_file(&mut self, path: &str) -> Result<()> {
        let path_buf = PathBuf::from(path);
        let buffer = Buffer::from_file(path)?;
        self.buffers.insert(path_buf, buffer);
        Ok(())
    }

    /// Apply a transaction to a shadow file
    pub fn apply_edit(&mut self, path: &str, tx: Transaction) -> Result<()> {
        let path_buf = PathBuf::from(path);
        if let Some(buffer) = self.buffers.get_mut(&path_buf) {
            buffer.apply(tx);
            Ok(())
        } else {
            Err(anyhow!("File not found in shadow workspace: {}", path))
        }
    }

    /// Get the content of a shadow file
    pub fn get_text(&self, path: &str) -> Option<String> {
        self.buffers.get(&PathBuf::from(path)).map(|b| b.text())
    }

    /// Check diagnostics (Mock implementation)
    /// In a real system, this would spawn `cargo check` or `tsc` on the shadow file content.
    pub fn check_diagnostics(&self, _path: &str) -> Vec<String> {
        // Mock: Check if "ERROR" string exists in content
        if let Some(text) = self.get_text(_path) {
            if text.contains("ERROR") {
                return vec!["Found explicit ERROR marker".to_string()];
            }
        }
        Vec::new()
    }
}

impl Default for ShadowWorkspace {
    fn default() -> Self {
        Self::new()
    }
}
