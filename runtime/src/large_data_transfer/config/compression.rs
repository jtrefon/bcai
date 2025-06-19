use serde::{Deserialize, Serialize};

/// Supported compression algorithms.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// No compression (pass-through)
    None,
    /// LZ4 — very fast, modest ratio
    Lz4,
    /// Zstd — slower but excellent ratio
    Zstd,
}

/// Compression configuration applied per transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable or disable compression globally.
    pub enabled: bool,
    /// Chosen compression algorithm.
    pub algorithm: CompressionAlgorithm,
    /// Quality / speed trade-off level (algorithm-specific).
    pub level: u32,
    /// Only compress payloads larger than this size (in bytes).
    pub min_size: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::Lz4,
            level: 4,     // Fast compression preset
            min_size: 1024, // 1 KB threshold
        }
    }
} 