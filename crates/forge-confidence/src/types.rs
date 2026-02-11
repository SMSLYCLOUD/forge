use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A field representing confidence scores for file paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceField {
    pub map: HashMap<String, f64>,
}

impl ConfidenceField {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: String, score: f64) {
        self.map.insert(path, score);
    }

    pub fn get(&self, path: &str) -> Option<&f64> {
        self.map.get(path)
    }
}

impl Default for ConfidenceField {
    fn default() -> Self {
        Self::new()
    }
}

/// The mode in which the confidence engine operates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceMode {
    Realtime,
    Background,
}
