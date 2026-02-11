pub struct TemporalEngine;

impl TemporalEngine {
    pub fn compute_decay(
        author_count: usize,
        patch_frequency: f64,
        churn_ratio: f64,
        days_since_commit: f64,
        initial_confidence: f64,
    ) -> f64 {
        let lambda = 0.1 * author_count as f64 + 0.05 * patch_frequency + 0.03 * churn_ratio;
        let decayed = initial_confidence * (-lambda * days_since_commit / 365.0).exp();
        decayed.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_decay_for_recent_code() {
        let score = TemporalEngine::compute_decay(1, 0.5, 0.1, 0.0, 0.95);
        assert!((score - 0.95).abs() < 1e-10); // e^0 = 1
    }

    #[test]
    fn high_decay_for_abandoned_code() {
        // 3 authors, 5 patches/month, 0.8 churn, 365 days
        // λ = 0.1*3 + 0.05*5 + 0.03*0.8 = 0.3 + 0.25 + 0.024 = 0.574
        // C = 0.95 * e^(-0.574 * 365/365) = 0.95 * e^(-0.574)
        // e^(-0.574) ≈ 0.563
        // C ≈ 0.95 * 0.563 ≈ 0.535
        let score = TemporalEngine::compute_decay(3, 5.0, 0.8, 365.0, 0.95);
        assert!(score < 0.60);
        assert!(score > 0.45);
        assert!((score - (0.95 * (-0.574f64).exp())).abs() < 1e-5);
    }
}
