//! Performance Optimizer for BCAI
//! 
//! This module provides performance optimization features including
//! caching, bandwidth management, resource allocation, and monitoring.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

/// Cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    size: u64,
}

impl CacheEntry {
    fn new(data: Vec<u8>) -> Self {
        let now = Instant::now();
        let size = data.len() as u64;
        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    fn access(&mut self) -> &Vec<u8> {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.data
    }
}

/// Bandwidth tracker for a connection
#[derive(Debug, Clone)]
struct BandwidthTracker {
    bytes_transferred: VecDeque<(Instant, u64)>,
    total_bytes: u64,
    start_time: Instant,
}

impl BandwidthTracker {
    fn new() -> Self {
        Self {
            bytes_transferred: VecDeque::new(),
            total_bytes: 0,
            start_time: Instant::now(),
        }
    }

    fn record_transfer(&mut self, bytes: u64) {
        let now = Instant::now();
        self.bytes_transferred.push_back((now, bytes));
        self.total_bytes += bytes;

        // Keep only last minute of data
        let cutoff = now - Duration::from_secs(60);
        while let Some(&(timestamp, _)) = self.bytes_transferred.front() {
            if timestamp < cutoff {
                self.bytes_transferred.pop_front();
            } else {
                break;
            }
        }
    }

    fn current_mbps(&self) -> f32 {
        if self.bytes_transferred.is_empty() {
            return 0.0;
        }

        let now = Instant::now();
        let window = Duration::from_secs(60);
        let cutoff = now - window;

        let bytes_in_window: u64 = self.bytes_transferred
            .iter()
            .filter(|(timestamp, _)| *timestamp >= cutoff)
            .map(|(_, bytes)| bytes)
            .sum();

        (bytes_in_window as f32 * 8.0) / (1_000_000.0 * 60.0) // Convert to Mbps
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

/// Performance optimizer
#[derive(Debug)]
pub struct PerformanceOptimizer {
    config: PerformanceConfig,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_size: Arc<RwLock<u64>>,
    bandwidth_trackers: Arc<RwLock<HashMap<String, BandwidthTracker>>>,
    resource_history: Arc<RwLock<VecDeque<ResourceMetrics>>>,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(config: PerformanceConfig) -> Self {
        let optimizer = Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_size: Arc::new(RwLock::new(0)),
            bandwidth_trackers: Arc::new(RwLock::new(HashMap::new())),
            resource_history: Arc::new(RwLock::new(VecDeque::new())),
        };

        // Start background monitoring if enabled
        if optimizer.config.enable_resource_monitoring {
            optimizer.start_resource_monitoring();
        }

        optimizer
    }

    /// Cache data with a key
    pub fn cache_put(&self, key: String, data: Vec<u8>) -> Result<(), PerformanceError> {
        if !self.config.enable_caching {
            return Ok(());
        }

        let entry = CacheEntry::new(data);
        let entry_size = entry.size;

        // Check if we need to evict entries
        self.ensure_cache_space(entry_size)?;

        {
            let mut cache = self.cache.write().unwrap();
            let mut cache_size = self.cache_size.write().unwrap();

            // Remove existing entry if it exists
            if let Some(old_entry) = cache.remove(&key) {
                *cache_size -= old_entry.size;
            }

            cache.insert(key, entry);
            *cache_size += entry_size;
        }

        Ok(())
    }

    /// Retrieve data from cache
    pub fn cache_get(&self, key: &str) -> Option<Vec<u8>> {
        if !self.config.enable_caching {
            return None;
        }

        let mut cache = self.cache.write().unwrap();
        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired(self.config.cache_ttl) {
                cache.remove(key);
                return None;
            }
            Some(entry.access().clone())
        } else {
            None
        }
    }

    /// Record bandwidth usage for a connection
    pub fn record_bandwidth(&self, connection_id: &str, bytes: u64) {
        if !self.config.enable_bandwidth_optimization {
            return;
        }

        let mut trackers = self.bandwidth_trackers.write().unwrap();
        let tracker = trackers.entry(connection_id.to_string())
            .or_insert_with(BandwidthTracker::new);
        tracker.record_transfer(bytes);
    }

    /// Check if connection is within bandwidth limits
    pub fn is_bandwidth_available(&self, connection_id: &str) -> bool {
        if !self.config.enable_bandwidth_optimization {
            return true;
        }

        let trackers = self.bandwidth_trackers.read().unwrap();
        if let Some(tracker) = trackers.get(connection_id) {
            tracker.current_mbps() < self.config.max_bandwidth_mbps as f32
        } else {
            true
        }
    }

    /// Get current bandwidth usage for a connection
    pub fn get_bandwidth_usage(&self, connection_id: &str) -> f32 {
        let trackers = self.bandwidth_trackers.read().unwrap();
        trackers.get(connection_id)
            .map(|tracker| tracker.current_mbps())
            .unwrap_or(0.0)
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        let cache = self.cache.read().unwrap();
        let cache_size = *self.cache_size.read().unwrap();
        let trackers = self.bandwidth_trackers.read().unwrap();
        let resource_history = self.resource_history.read().unwrap();

        let cache_entries = cache.len();
        let cache_hit_rate = if cache_entries > 0 {
            let total_accesses: u64 = cache.values().map(|e| e.access_count).sum();
            if total_accesses > 0 {
                cache_entries as f32 / total_accesses as f32
            } else {
                0.0
            }
        } else {
            0.0
        };

        let active_connections = trackers.len();
        let total_bandwidth: f32 = trackers.values().map(|t| t.current_mbps()).sum();

        let latest_metrics = resource_history.back().cloned().unwrap_or_default();

        PerformanceStats {
            cache_entries,
            cache_size_bytes: cache_size,
            cache_hit_rate,
            active_connections,
            total_bandwidth_mbps: total_bandwidth,
            cpu_usage_percent: latest_metrics.cpu_usage_percent,
            memory_usage_percent: latest_metrics.memory_usage_percent,
            disk_usage_percent: latest_metrics.disk_usage_percent,
            optimization_enabled: self.config.enable_caching && self.config.enable_bandwidth_optimization,
        }
    }

    /// Ensure there's enough cache space for new entry
    fn ensure_cache_space(&self, needed_size: u64) -> Result<(), PerformanceError> {
        let current_size = *self.cache_size.read().unwrap();
        
        if current_size + needed_size <= self.config.max_cache_size {
            return Ok(());
        }

        // Need to evict entries - use LRU strategy
        let mut cache = self.cache.write().unwrap();
        let mut cache_size = self.cache_size.write().unwrap();

        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.last_accessed);

        let mut freed_space = 0u64;
        let mut keys_to_remove = Vec::new();

        for (key, entry) in entries {
            keys_to_remove.push(key.clone());
            freed_space += entry.size;
            
            if *cache_size - freed_space + needed_size <= self.config.max_cache_size {
                break;
            }
        }

        for key in keys_to_remove {
            if let Some(entry) = cache.remove(&key) {
                *cache_size -= entry.size;
            }
        }

        if *cache_size + needed_size > self.config.max_cache_size {
            return Err(PerformanceError::CacheFull);
        }

        Ok(())
    }

    /// Start background resource monitoring
    fn start_resource_monitoring(&self) {
        let resource_history = Arc::clone(&self.resource_history);
        let interval = self.config.monitoring_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Simulate resource collection (in reality would use system APIs)
                let metrics = ResourceMetrics {
                    cpu_usage_percent: rand::random::<f32>() * 100.0,
                    memory_usage_bytes: (rand::random::<f32>() * 8.0 * 1024.0 * 1024.0 * 1024.0) as u64,
                    memory_usage_percent: rand::random::<f32>() * 100.0,
                    disk_usage_bytes: (rand::random::<f32>() * 100.0 * 1024.0 * 1024.0 * 1024.0) as u64,
                    disk_usage_percent: rand::random::<f32>() * 100.0,
                    network_rx_mbps: rand::random::<f32>() * 1000.0,
                    network_tx_mbps: rand::random::<f32>() * 1000.0,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                };

                {
                    let mut history = resource_history.write().unwrap();
                    history.push_back(metrics);
                    
                    // Keep only last hour of data (360 entries at 10s intervals)
                    while history.len() > 360 {
                        history.pop_front();
                    }
                }
            }
        });
    }
}

/// Performance errors
#[derive(Debug, Clone, thiserror::Error)]
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

/// Performance statistics
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_optimizer_creation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.cache_entries, 0);
        assert_eq!(stats.active_connections, 0);
    }

    #[tokio::test]
    async fn test_caching() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let key = "test_key".to_string();
        let data = b"test data".to_vec();
        
        optimizer.cache_put(key.clone(), data.clone()).unwrap();
        let retrieved = optimizer.cache_get(&key);
        
        assert_eq!(retrieved, Some(data));
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.cache_entries, 1);
    }

    #[tokio::test]
    async fn test_bandwidth_tracking() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        let connection_id = "conn1";
        
        assert!(optimizer.is_bandwidth_available(connection_id));
        
        optimizer.record_bandwidth(connection_id, 1024);
        let usage = optimizer.get_bandwidth_usage(connection_id);
        
        assert!(usage >= 0.0);
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.active_connections, 1);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let mut config = PerformanceConfig::default();
        config.max_cache_size = 100; // Very small cache
        
        let optimizer = PerformanceOptimizer::new(config);
        
        // Fill cache beyond capacity
        for i in 0..10 {
            let key = format!("key_{}", i);
            let data = vec![0u8; 50]; // 50 bytes each
            optimizer.cache_put(key, data).unwrap();
        }
        
        let stats = optimizer.get_stats();
        assert!(stats.cache_size_bytes <= 100);
    }
} 