use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TemporalDecay {
    // Duration in seconds for serialization simplicity, or just use Duration
    // helper field
    max_age_seconds: i64,
}

impl TemporalDecay {
    pub fn new(days: i64) -> Self {
        Self {
            max_age_seconds: days * 24 * 3600,
        }
    }

    pub fn apply(&self, confidence: f64, last_verified: DateTime<Utc>) -> f64 {
        let age = Utc::now().signed_duration_since(last_verified);
        let age_seconds = age.num_seconds();

        if age_seconds >= self.max_age_seconds {
            return 0.0;
        }

        if age_seconds < 0 {
            // Clock skew or future timestamp
            return confidence;
        }

        // Linear decay
        let remaining_life = 1.0 - (age_seconds as f64 / self.max_age_seconds as f64);
        confidence * remaining_life
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_temporal_decay() {
        let decay = TemporalDecay::new(30); // 30 days
        let now = Utc::now();

        // Just verified: 1.0
        assert!((decay.apply(1.0, now) - 1.0).abs() < 1e-6);

        // 15 days ago: 0.5
        let past = now - Duration::days(15);
        assert!((decay.apply(1.0, past) - 0.5).abs() < 1e-6);

        // 30 days ago: 0.0
        let expired = now - Duration::days(30);
        assert!(decay.apply(1.0, expired) <= 1e-6);

        // 31 days ago: 0.0
        let old = now - Duration::days(31);
        assert_eq!(decay.apply(1.0, old), 0.0);
    }
}
