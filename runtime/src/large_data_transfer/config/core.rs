use super::{CacheConfig, CompressionConfig, EncryptionConfig, RetryConfig};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Top-level configuration aggregating all sub-configs required by the
/// large-data-transfer subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeDataConfig {
    /// Default chunk size in bytes (1MB to 4MB recommended)
    pub default_chunk_size: u32,
    /// Maximum number of concurrent transfers
    pub max_concurrent_transfers: usize,
    /// Maximum upload rate in bytes per second (0 = unlimited)
    pub max_upload_rate: u64,
    /// Maximum download rate in bytes per second (0 = unlimited)
    pub max_download_rate: u64,
    /// Timeout for individual chunk requests
    pub chunk_timeout: Duration,
    /// Timeout for entire transfer operations
    pub transfer_timeout: Duration,
    /// Local cache configuration
    pub cache_config: CacheConfig,
    /// Compression configuration
    pub compression_config: CompressionConfig,
    /// Encryption configuration
    pub encryption_config: EncryptionConfig,
    /// Retry strategy when chunks fail
    pub retry_config: RetryConfig,
    /// Interval for peer updates (e.g. refresh peer stats).
    pub peer_update_interval: Duration,
    /// Timeout after which a peer is considered stale and removed.
    pub peer_timeout: Duration,
}

impl Default for LargeDataConfig {
    fn default() -> Self {
        Self {
            default_chunk_size: 2 * 1024 * 1024,          // 2 MB
            max_concurrent_transfers: 10,
            max_upload_rate: 0,                           // Unlimited
            max_download_rate: 0,                         // Unlimited
            chunk_timeout: Duration::from_secs(30),
            transfer_timeout: Duration::from_secs(3600),  // 1 hour
            cache_config: CacheConfig::default(),
            compression_config: CompressionConfig::default(),
            encryption_config: EncryptionConfig::default(),
            retry_config: RetryConfig::default(),
            peer_update_interval: Duration::from_secs(30),
            peer_timeout: Duration::from_secs(300),
        }
    }
} 