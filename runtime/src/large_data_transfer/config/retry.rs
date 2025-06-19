use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for retry/back-off strategy when chunk transfers fail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts before giving up.
    pub max_retries: u32,
    /// Delay before the first retry attempt.
    pub initial_delay: Duration,
    /// Exponential back-off multiplier.
    pub backoff_multiplier: f32,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Random jitter factor (0.0–1.0) to avoid thundering-herd.
    pub jitter: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(60),
            jitter: 0.1,
        }
    }
} 