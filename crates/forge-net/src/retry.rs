use std::time::Duration;
use rand::Rng;

/// Retry policy with exponential backoff and decorrelated jitter
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub max_attempts: u32,
}

impl RetryPolicy {
    pub fn default_policy() -> Self {
        Self {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            max_attempts: 5,
        }
    }

    /// Decorrelated jitter: min(cap, random(base, prev_delay * 3))
    pub fn next_delay(&self, _attempt: u32, prev_delay: Duration) -> Duration {
        let max_ms = self.max_delay.as_millis() as u64;
        let base_ms = self.base_delay.as_millis() as u64;
        let prev_ms = prev_delay.as_millis() as u64;
        let upper = prev_ms.saturating_mul(3).min(max_ms);
        if upper <= base_ms {
            return self.base_delay;
        }
        let delay_ms = rand::thread_rng().gen_range(base_ms..=upper);
        Duration::from_millis(delay_ms)
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::default_policy()
    }
}
