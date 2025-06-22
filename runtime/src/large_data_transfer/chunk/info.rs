//! Defines the `ChunkInfo` struct, which contains metadata about a chunk.

use super::id::ChunkId;
use crate::large_data_transfer::config::CompressionAlgorithm;
use serde::{Deserialize, Serialize};

/// Metadata about a data chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Unique chunk identifier (hash of the data).
    pub id: ChunkId,

    /// Original size of the data before compression.
    pub original_size: u32,

    /// Compressed size of the data (0 if not compressed).
    pub compressed_size: u32,

    /// The compression algorithm used on the data.
    pub compression: CompressionAlgorithm,

    /// CRC32 checksum for fast integrity verification of the original data.
    pub checksum: u32,

    /// The sequence index of this chunk in the original data object.
    pub index: u32,

    /// Node IDs that currently store a replica of this chunk (original holder + copies).
    #[serde(default)]
    pub replicas: Vec<String>,
}

impl ChunkInfo {
    /// Calculate the compression ratio (compressed_size / original_size).
    pub fn compression_ratio(&self) -> f32 {
        if self.original_size == 0 {
            return 1.0;
        }

        let compressed = if self.compressed_size > 0 {
            self.compressed_size
        } else {
            self.original_size
        };

        compressed as f32 / self.original_size as f32
    }

    /// Returns true if the chunk is compressed.
    pub fn is_compressed(&self) -> bool {
        self.compression != CompressionAlgorithm::None && self.compressed_size > 0
    }
} 