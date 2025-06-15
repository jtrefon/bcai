//! Large Data Transfer Module
//!
//! This module provides efficient transfer of large data blocks (GB to TB scale)
//! through chunking, content addressing, compression, and streaming protocols.
//!
//! Key features:
//! - Content-addressed chunking with integrity verification
//! - Streaming transfer protocol with flow control
//! - Bandwidth management and QoS controls
//! - Local caching with deduplication
//! - Encryption and security features

pub mod chunk;
pub mod descriptor;
pub mod manager;
pub mod protocol;
pub mod cache;
pub mod compression;
pub mod crypto;

// Re-export core types
pub use chunk::{DataChunk, ChunkId, ChunkInfo};
pub use descriptor::{LargeDataDescriptor, TransferMetadata};
pub use manager::{ChunkManager, ChunkManagerConfig};
pub use protocol::{TransferMessage, TransferState, TransferError};

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

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

/// Transfer priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransferPriority {
    /// Low priority - background transfers
    Low = 0,
    /// Normal priority - default
    Normal = 1,
    /// High priority - urgent transfers
    High = 2,
    /// Critical priority - system-critical transfers
    Critical = 3,
}

impl Default for TransferPriority {
    fn default() -> Self {
        TransferPriority::Normal
    }
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

/// Large data transfer errors
#[derive(Error, Debug)]
pub enum LargeDataError {
    #[error("Chunk error: {0}")]
    Chunk(#[from] chunk::ChunkError),
    
    #[error("Compression error: {0}")]
    Compression(String),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type for large data operations
pub type LargeDataResult<T> = Result<T, LargeDataError>;

/// Transfer statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferStats {
    /// Total bytes transferred
    pub bytes_transferred: u64,
    
    /// Current transfer rate in bytes per second
    pub transfer_rate: f64,
    
    /// Chunks successfully transferred
    pub chunks_completed: u32,
    
    /// Total chunks in transfer
    pub total_chunks: u32,
    
    /// Transfer completion percentage (0.0 to 1.0)
    pub completion_percentage: f32,
    
    /// Estimated time to completion
    pub eta: Option<Duration>,
    
    /// Number of retry attempts
    pub retry_count: u32,
    
    /// Current active connections
    pub active_connections: u32,
    
    /// Compression ratio achieved
    pub compression_ratio: f32,
    
    /// Cache hit rate
    pub cache_hit_rate: f32,
}

impl Default for TransferStats {
    fn default() -> Self {
        Self {
            bytes_transferred: 0,
            transfer_rate: 0.0,
            chunks_completed: 0,
            total_chunks: 0,
            completion_percentage: 0.0,
            eta: None,
            retry_count: 0,
            active_connections: 0,
            compression_ratio: 1.0,
            cache_hit_rate: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LargeDataConfig::default();
        assert_eq!(config.default_chunk_size, 2 * 1024 * 1024);
        assert_eq!(config.max_concurrent_transfers, 10);
        assert!(config.cache_config.deduplication);
        assert!(config.compression_config.enabled);
        assert!(!config.encryption_config.enabled);
    }

    #[test]
    fn test_transfer_priority_ordering() {
        assert!(TransferPriority::Critical > TransferPriority::High);
        assert!(TransferPriority::High > TransferPriority::Normal);
        assert!(TransferPriority::Normal > TransferPriority::Low);
    }

    #[test]
    fn test_compression_algorithms() {
        let alg = CompressionAlgorithm::Lz4;
        assert_eq!(alg, CompressionAlgorithm::Lz4);
        assert_ne!(alg, CompressionAlgorithm::Zstd);
    }

    #[test]
    fn test_transfer_stats_default() {
        let stats = TransferStats::default();
        assert_eq!(stats.bytes_transferred, 0);
        assert_eq!(stats.completion_percentage, 0.0);
        assert_eq!(stats.compression_ratio, 1.0);
    }
} 