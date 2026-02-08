use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use crate::ConnectionState;

/// Background health monitor
pub struct HealthMonitor {
    state: Arc<RwLock<ConnectionState>>,
    check_interval: Duration,
}

impl HealthMonitor {
    pub fn new(state: Arc<RwLock<ConnectionState>>, check_interval: Duration) -> Self {
        Self { state, check_interval }
    }

    /// Run health check loop (call from tokio spawn)
    pub async fn run(&self) {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        loop {
            tokio::time::sleep(self.check_interval).await;

            let start = Instant::now();
            let result = client
                .head("https://httpbin.org/status/200")
                .send()
                .await;

            let new_state = match result {
                Ok(resp) if resp.status().is_success() => {
                    let latency = start.elapsed().as_millis() as u32;
                    ConnectionState::Online { latency_ms: latency }
                }
                Ok(_) => ConnectionState::Degraded { error_rate: 0.5 },
                Err(_) => ConnectionState::Offline {
                    since: Instant::now(),
                    queued: 0,
                },
            };

            if let Ok(mut state) = self.state.try_write() {
                *state = new_state;
            }
        }
    }
}
