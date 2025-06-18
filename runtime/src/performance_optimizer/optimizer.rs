use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable caching
    pub enable_caching: bool,
    /// Maximum cache size (bytes)
    pub max_cache_size: u64,
    /// Cache TTL (time to live)
    pub cache_ttl: Duration,
    /// Enable bandwidth optimization
    pub enable_bandwidth_optimization: bool,
    /// Maximum bandwidth per connection (Mbps)
    pub max_bandwidth_mbps: u32,
    /// Enable resource monitoring
    pub enable_resource_monitoring: bool,
    /// Monitoring interval
    pub monitoring_interval: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_cache_size: 1024 * 1024 * 1024, // 1GB
            cache_ttl: Duration::from_secs(3600), // 1 hour
            enable_bandwidth_optimization: true,
            max_bandwidth_mbps: 100,
            enable_resource_monitoring: true,
            monitoring_interval: Duration::from_secs(10),
        }
    }
} 