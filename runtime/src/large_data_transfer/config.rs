//! Configuration structures for the large data transfer module.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for large data transfers
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
}

impl Default for LargeDataConfig {
    fn default() -> Self {
        Self {
            default_chunk_size: 2 * 1024 * 1024, // 2MB
            max_concurrent_transfers: 10,
            max_upload_rate: 0, // Unlimited
            max_download_rate: 0, // Unlimited
            chunk_timeout: Duration::from_secs(30),
            transfer_timeout: Duration::from_secs(3600), // 1 hour
            cache_config: CacheConfig::default(),
            compression_config: CompressionConfig::default(),
            encryption_config: EncryptionConfig::default(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in bytes (default: 10GB)
    pub max_size: u64,

    /// Enable LRU eviction
    pub lru_eviction: bool,

    /// Enable deduplication
    pub deduplication: bool,

    /// Cache directory path
    pub cache_dir: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024 * 1024, // 10GB
            lru_eviction: true,
            deduplication: true,
            cache_dir: None, // Use system temp dir
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,

    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,

    /// Compression level (algorithm-specific)
    pub level: u32,

    /// Minimum size to trigger compression (bytes)
    pub min_size: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::Lz4,
            level: 4, // Fast compression
            min_size: 1024, // 1KB
        }
    }
}

/// Supported compression algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// LZ4 - Fast compression/decompression
    Lz4,
    /// Zstd - Better compression ratio
    Zstd,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    pub enabled: bool,

    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,

    /// Encrypt individual chunks (vs stream-level encryption)
    pub chunk_encryption: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
            chunk_encryption: false,
        }
    }
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    /// No encryption
    None,
    /// ChaCha20-Poly1305 AEAD
    ChaCha20Poly1305,
    /// AES-256-GCM AEAD
    Aes256Gcm,
}

/// Retry strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial retry delay
    pub initial_delay: Duration,

    /// Backoff multiplier (exponential backoff)
    pub backoff_multiplier: f32,

    /// Maximum retry delay
    pub max_delay: Duration,

    /// Jitter factor (0.0 to 1.0) to avoid thundering herd
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