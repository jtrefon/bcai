use super::firewall::RateLimitConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub enable_authentication: bool,
    /// Enable encryption for data at rest
    pub enable_encryption_at_rest: bool,
    /// Enable encryption for data in transit
    pub enable_encryption_in_transit: bool,
    /// Session timeout duration
    pub session_timeout: Duration,
    /// Maximum failed authentication attempts
    pub max_auth_attempts: u32,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_authentication: true,
            enable_encryption_at_rest: true,
            enable_encryption_in_transit: true,
            session_timeout: Duration::from_secs(3600), // 1 hour
            max_auth_attempts: 3,
            rate_limit: RateLimitConfig::default(),
        }
    }
} 