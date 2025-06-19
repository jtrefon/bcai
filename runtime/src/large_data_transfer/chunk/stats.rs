//! Defines statistics related to chunk compression.

use crate::large_data_transfer::config::CompressionAlgorithm;
use serde::{Deserialize, Serialize};

/// Compression statistics for a chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkStats {
    pub algorithm: CompressionAlgorithm,
    pub original_size: u32,
    pub compressed_size: u32,
    pub ratio: f32,
    pub savings_bytes: u32,
}

impl ChunkStats {
    /// The percentage of space saved by compression.
    pub fn savings_percentage(&self) -> f32 {
        if self.original_size == 0 {
            return 0.0;
        }
        (self.savings_bytes as f32 / self.original_size as f32) * 100.0
    }
}

// Backward-compat alias
pub type CompressionStats = ChunkStats; 