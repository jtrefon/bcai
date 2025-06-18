//! Defines the configuration for the ChunkManager.

use crate::large_data_transfer::config::LargeDataConfig;
use std::time::Duration;

/// Configuration for the chunk manager.
#[derive(Debug, Clone)]
pub struct ChunkManagerConfig {
    /// Maximum number of chunks to keep in memory.
    pub max_memory_chunks: usize,

    /// Maximum total memory usage in bytes for all chunks.
    pub max_memory_bytes: u64,

    /// Interval at which to run the cleanup task for expired chunks.
    pub cleanup_interval: Duration,

    /// Default time-to-live for a chunk in the cache.
    pub default_expiration: Duration,
}

impl Default for ChunkManagerConfig {
    fn default() -> Self {
        Self {
            max_memory_chunks: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            cleanup_interval: Duration::from_secs(60),
            default_expiration: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl From<&LargeDataConfig> for ChunkManagerConfig {
    fn from(config: &LargeDataConfig) -> Self {
        // Here you might derive the chunk manager config from the main data transfer config.
        // For now, we use default values but this provides a hook for future integration.
        Self::default()
    }
} 