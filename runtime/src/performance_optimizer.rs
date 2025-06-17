//! Performance Optimizer for BCAI
//! 
//! This module provides performance optimization features including
//! caching, bandwidth management, resource allocation, and monitoring.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
    pub memory_usage_percent: f32,
    pub disk_usage_bytes: u64,
    pub disk_usage_percent: f32,
    pub network_rx_mbps: f32,
    pub network_tx_mbps: f32,
    pub timestamp: u64,
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            memory_usage_percent: 0.0,
            disk_usage_bytes: 0,
            disk_usage_percent: 0.0,
            network_rx_mbps: 0.0,
            network_tx_mbps: 0.0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PerformanceError {
    #[error("Cache is full")]
    CacheFull,
    #[error("Bandwidth limit exceeded")]
    BandwidthLimitExceeded,
    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,
    #[error("Optimization error: {0}")]
    OptimizationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub cache_entries: usize,
    pub cache_size_bytes: u64,
    pub cache_hit_rate: f32,
    pub active_connections: usize,
    pub total_bandwidth_mbps: f32,
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub disk_usage_percent: f32,
    pub optimization_enabled: bool,
}

// NOTE: Removed placeholder implementation structs:
// - PerformanceOptimizer
// - CacheEntry
// - BandwidthTracker
// This file now only defines the data models for the performance optimizer. 