use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::markov::MarkovChain;

const GHOST_TAB_THRESHOLD: f64 = 0.3;
const MAX_GHOST_TABS: usize = 3;

#[derive(Debug, Serialize, Deserialize)]
pub struct GhostTabsEngine {
    chain: MarkovChain,
    current_file: Option<String>,
}

impl Default for GhostTabsEngine {
    fn default() -> Self {
        Self {
            chain: MarkovChain::new(),
            current_file: None,
        }
    }
}

impl GhostTabsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records that a file was opened.
    /// Updates the Markov chain based on the transition from the previous file.
    pub fn on_file_open(&mut self, file_path: &str) {
        if let Some(prev) = &self.current_file {
            self.chain.record_transition(prev, file_path);
        }
        self.current_file = Some(file_path.to_string());
    }

    /// Returns a list of predicted ghost tabs based on the current file.
    /// Only returns files with probability > 0.3.
    pub fn get_suggestions(&self) -> Vec<String> {
        let Some(current) = &self.current_file else {
            return Vec::new();
        };

        self.chain
            .predict(current, MAX_GHOST_TABS)
            .into_iter()
            .filter(|&(_, prob)| prob > GHOST_TAB_THRESHOLD)
            .map(|(file, _)| file)
            .collect()
    }

    /// Saves the engine state to a JSON file.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Loads the engine state from a JSON file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let engine = serde_json::from_str(&content)?;
        Ok(engine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_tabs_prediction() {
        let mut engine = GhostTabsEngine::new();

        // Sequence: A -> B -> A -> B -> A -> C
        engine.on_file_open("A");
        engine.on_file_open("B"); // A->B
        engine.on_file_open("A"); // B->A
        engine.on_file_open("B"); // A->B
        engine.on_file_open("A"); // B->A
        engine.on_file_open("C"); // A->C

        // Current file is C. Transitions from C are none.
        let suggestions = engine.get_suggestions();
        assert!(suggestions.is_empty());

        // Switch to A.
        engine.on_file_open("A"); // C->A
        // Transitions from A: B (2 times), C (1 time). Total 3.
        // P(B|A) = 2/3 = 0.66 > 0.3 -> Suggest B
        // P(C|A) = 1/3 = 0.33 > 0.3 -> Suggest C

        let suggestions = engine.get_suggestions();
        assert!(suggestions.contains(&"B".to_string()));
        assert!(suggestions.contains(&"C".to_string()));
        assert_eq!(suggestions.len(), 2);
    }
}
