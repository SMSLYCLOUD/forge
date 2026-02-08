use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum ActionType {
    DismissWarning,
    FixWarning,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnomalyDetector {
    window_size: usize,
    history: VecDeque<ActionType>,
}

impl AnomalyDetector {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            history: VecDeque::with_capacity(window_size),
        }
    }

    pub fn record_action(&mut self, action: ActionType) {
        if self.history.len() >= self.window_size {
            self.history.pop_front();
        }
        self.history.push_back(action);
    }

    pub fn is_anomalous(&self) -> bool {
        if self.history.len() < self.window_size / 2 {
            // Need at least half window size to make a judgment
            return false;
        }
        let dismissals = self.history.iter().filter(|&&a| a == ActionType::DismissWarning).count();
        (dismissals as f64 / self.history.len() as f64) > 0.8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_detection() {
        let mut detector = AnomalyDetector::new(10);

        // Add 5 dismissals (50% full, all dismissals) -> 100% rate -> Anomalous
        for _ in 0..5 {
            detector.record_action(ActionType::DismissWarning);
        }
        assert!(detector.is_anomalous());

        // Add 5 fixes -> 50% rate -> Not anomalous
        for _ in 0..5 {
            detector.record_action(ActionType::FixWarning);
        }
        // Total 10 actions: 5 dismiss, 5 fix. 50% dismiss.
        assert!(!detector.is_anomalous());

        // Add 5 more dismissals (pushes out first 5 dismissals?)
        // History was [D, D, D, D, D, F, F, F, F, F]
        // Push D -> [D, D, D, D, F, F, F, F, F, D] (still 50%)
        // Push 5 D -> [F, F, F, F, F, D, D, D, D, D] (50%)
        // Push 5 more D -> [D, D, D, D, D, D, D, D, D, D] (100%)

        for _ in 0..10 {
            detector.record_action(ActionType::DismissWarning);
        }
        assert!(detector.is_anomalous());
    }
}
