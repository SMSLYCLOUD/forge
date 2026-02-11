use crate::health::HealthMonitor;
use crate::retry::RetryPolicy;
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Connection state for status bar display
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Online { latency_ms: u32 },
    Degraded { error_rate: f32 },
    Offline { since: Instant, queued: usize },
    Reconnecting { attempt: u32 },
}

impl ConnectionState {
    pub fn display(&self) -> String {
        match self {
            Self::Online { latency_ms } => format!("🌐 Online {}ms", latency_ms),
            Self::Degraded { error_rate } => format!("⚠️ Degraded {:.0}%", error_rate * 100.0),
            Self::Offline { queued, .. } => format!("🔴 Offline ({} queued)", queued),
            Self::Reconnecting { attempt } => format!("🔄 Retry #{}", attempt),
        }
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Online { latency_ms: 0 }
    }
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetConfig {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub health_check_interval: Duration,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_retries: 5,
            health_check_interval: Duration::from_secs(5),
        }
    }
}

/// The network client with auto-retry and health monitoring
pub struct ForgeNet {
    client: reqwest::Client,
    retry_policy: RetryPolicy,
    pub state: Arc<RwLock<ConnectionState>>,
}

impl ForgeNet {
    pub fn new(config: NetConfig) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(20)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .gzip(true)
            .brotli(true)
            .tcp_nodelay(true)
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        let state = Arc::new(RwLock::new(ConnectionState::default()));

        // Start health monitor
        let health_state = state.clone();
        let health_interval = config.health_check_interval;
        tokio::spawn(async move {
            let monitor = HealthMonitor::new(health_state, health_interval);
            monitor.run().await;
        });

        Self {
            client,
            retry_policy: RetryPolicy::default(),
            state,
        }
    }

    /// Send a request with automatic retry
    pub async fn request(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let mut attempt = 0u32;
        let mut prev_delay = self.retry_policy.base_delay;

        loop {
            // We need to clone the request for each attempt
            // Unfortunately reqwest::RequestBuilder doesn't implement Clone
            // So the caller should pass a closure or use this simpler approach
            match request.try_clone() {
                Some(req) => {
                    match req.send().await {
                        Ok(resp) if resp.status().is_success() => {
                            return Ok(resp);
                        }
                        Ok(resp) if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS => {
                            // Rate limited
                            let retry_after = resp
                                .headers()
                                .get("retry-after")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|v| v.parse::<u64>().ok())
                                .unwrap_or(1);
                            tokio::time::sleep(Duration::from_secs(retry_after)).await;
                            continue;
                        }
                        Ok(resp) => {
                            // Non-retryable error status
                            return Ok(resp);
                        }
                        Err(e) if self.retry_policy.should_retry(attempt) => {
                            attempt += 1;
                            let delay = self.retry_policy.next_delay(attempt, prev_delay);
                            prev_delay = delay;
                            tracing::warn!(
                                "Request failed (attempt {}): {}, retrying in {:?}",
                                attempt,
                                e,
                                delay
                            );
                            tokio::time::sleep(delay).await;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                None => {
                    anyhow::bail!("Request cannot be retried (body was already consumed)");
                }
            }
        }
    }

    /// Get current connection state (non-blocking, for UI)
    pub fn current_state(&self) -> ConnectionState {
        self.state.try_read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Get the underlying reqwest client for direct use
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
