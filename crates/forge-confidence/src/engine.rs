use crate::models::{ConfidenceScore, Signal};

pub struct ConfidenceEngine;

impl ConfidenceEngine {
    pub fn compute_line(&self, signals: &[Signal]) -> f64 {
        let available: Vec<_> = signals.iter().filter(|s| s.available).collect();
        let total_weight: f64 = available.iter().map(|s| s.weight).sum();

        if total_weight == 0.0 {
            return 0.5;
        }

        available
            .iter()
            .map(|s| (s.weight / total_weight) * s.value)
            .sum::<f64>()
            .clamp(0.0, 1.0)
    }

    /// CVaR at 10% — average of worst 10% of line scores
    pub fn compute_file(&self, lines: &[ConfidenceScore]) -> f64 {
        if lines.is_empty() {
            return 0.5;
        }
        let mut scores: Vec<f64> = lines.iter().map(|l| l.score).collect();
        scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = (scores.len() as f64 * 0.10).ceil().max(1.0) as usize;
        scores[..n].iter().sum::<f64>() / n as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_perfect_signals() {
        use crate::models::SignalKind;
        let engine = ConfidenceEngine;
        let signals = vec![
            Signal {
                name: SignalKind::SyntaxValid,
                value: 1.0,
                weight: 0.20,
                available: true,
            },
            Signal {
                name: SignalKind::LintClean,
                value: 1.0,
                weight: 0.10,
                available: true,
            },
        ];
        // Weight sum = 0.3. Redistributed: 0.2/0.3 * 1 + 0.1/0.3 * 1 = 1.0
        assert!((engine.compute_line(&signals) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn no_signals_returns_half() {
        let engine = ConfidenceEngine;
        assert!((engine.compute_line(&[]) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn cvar_worst_ten_percent() {
        let engine = ConfidenceEngine;
        let lines: Vec<ConfidenceScore> = (1..=10)
            .map(|i| ConfidenceScore::new(i, i as f64 * 0.1, vec![]))
            .collect();
        // Scores: 0.1, 0.2, ... 1.0. 10 items. 10% = 1 item.
        // Worst 1 item is 0.1. Average is 0.1.
        assert!((engine.compute_file(&lines) - 0.1).abs() < 1e-10);
    }
}
