use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarkovChain {
    // Maps current_file -> (next_file -> count)
    transitions: HashMap<String, HashMap<String, u32>>,
    // Total transitions from current_file
    totals: HashMap<String, u32>,
}

impl MarkovChain {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a transition from one file to another.
    pub fn record_transition(&mut self, from: &str, to: &str) {
        if from == to {
            return; // Ignore self-transitions (e.g. refocusing same file)
        }

        let entry = self
            .transitions
            .entry(from.to_string())
            .or_default();

        *entry.entry(to.to_string()).or_insert(0) += 1;

        *self.totals.entry(from.to_string()).or_insert(0) += 1;
    }

    /// Predicts the next most likely files given the current file.
    /// Returns a list of (file, probability) tuples, sorted by probability descending.
    pub fn predict(&self, current: &str, top_n: usize) -> Vec<(String, f64)> {
        let Some(next_counts) = self.transitions.get(current) else {
            return Vec::new();
        };

        let total = self.totals.get(current).copied().unwrap_or(0) as f64;
        if total == 0.0 {
            return Vec::new();
        }

        let mut predictions: Vec<(String, f64)> = next_counts
            .iter()
            .map(|(file, count)| (file.clone(), *count as f64 / total))
            .collect();

        // Sort by probability descending
        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        predictions.into_iter().take(top_n).collect()
    }

    /// Returns the transition matrix size (number of entries).
    pub fn len(&self) -> usize {
        self.transitions.values().map(|m| m.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markov_chain() {
        let mut chain = MarkovChain::new();

        // A -> B (3 times)
        chain.record_transition("A", "B");
        chain.record_transition("A", "B");
        chain.record_transition("A", "B");

        // A -> C (1 time)
        chain.record_transition("A", "C");

        // B -> A (1 time)
        chain.record_transition("B", "A");

        let preds_a = chain.predict("A", 3);
        assert_eq!(preds_a.len(), 2);
        assert_eq!(preds_a[0].0, "B");
        assert!((preds_a[0].1 - 0.75).abs() < 1e-6);
        assert_eq!(preds_a[1].0, "C");
        assert!((preds_a[1].1 - 0.25).abs() < 1e-6);

        let preds_b = chain.predict("B", 3);
        assert_eq!(preds_b.len(), 1);
        assert_eq!(preds_b[0].0, "A");
        assert!((preds_b[0].1 - 1.0).abs() < 1e-6);

        let preds_c = chain.predict("C", 3);
        assert!(preds_c.is_empty());
    }
}
