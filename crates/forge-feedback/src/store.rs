use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackStore {
    pub priors: HashMap<String, f64>,
    pub action_count: usize,
}

impl FeedbackStore {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self {
                priors: HashMap::new(),
                action_count: 0,
            })
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
