//! Data Chunk Implementation
//!
//! This module provides the core data chunking functionality for large data transfers.
//! Chunks are content-addressed, compressed, and integrity-verified.

use crate::large_data_transfer::{CompressionAlgorithm, LargeDataResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;

/// Unique identifier for a data chunk (SHA-256 hash)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkId(pub String);

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0[..8]) // Show first 8 characters
    }
}

impl ChunkId {
    /// Create a new ChunkId from data
    pub fn from_data(data: &[u8]) -> Self {
        let hash = Sha256::digest(data);
        Self(format!("{:x}", hash))
    }

    /// Create ChunkId from hex string
    pub fn from_hex(hex: &str) -> Result<Self, ChunkError> {
        if hex.len() != 64 {
            return Err(ChunkError::InvalidHash("Hash must be 64 hex characters".to_string()));
        }
        
        // Validate hex format
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ChunkError::InvalidHash("Invalid hex characters".to_string()));
        }
        
        Ok(Self(hex.to_string()))
    }

    /// Get the full hash string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get short hash (first 8 characters) for display
    pub fn short_hash(&self) -> &str {
        &self.0[..8]
    }
}

/// Information about a data chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Unique chunk identifier
    pub id: ChunkId,
    
    /// Original size before compression
    pub original_size: u32,
    
    /// Compressed size (0 if not compressed)
    pub compressed_size: u32,
    
    /// Compression algorithm used
    pub compression: CompressionAlgorithm,
    
    /// CRC32 checksum for fast integrity verification
    pub checksum: u32,
    
    /// Chunk sequence index in the original data
    pub index: u32,
}

impl ChunkInfo {
    /// Calculate compression ratio (0.0 to 1.0, lower is better)
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

    /// Check if chunk is compressed
    pub fn is_compressed(&self) -> bool {
        self.compression != CompressionAlgorithm::None && self.compressed_size > 0
    }
}

/// A data chunk with content-addressing and compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChunk {
    /// Chunk metadata
    pub info: ChunkInfo,
    
    /// Actual chunk data (may be compressed)
    pub data: Vec<u8>,
}

impl DataChunk {
    /// Create a new data chunk from raw data
    pub fn new(
        data: Vec<u8>,
        index: u32,
        compression: CompressionAlgorithm,
    ) -> LargeDataResult<Self> {
        let original_size = data.len() as u32;
        let checksum = crc32fast::hash(&data);
        
        // Compress data if requested
        let (final_data, compressed_size) = match compression {
            CompressionAlgorithm::None => (data, 0),
            CompressionAlgorithm::Lz4 => {
                let compressed = lz4_flex::compress_prepend_size(&data);
                if compressed.len() < data.len() {
                    let compressed_size = compressed.len() as u32;
                    (compressed, compressed_size)
                } else {
                    // Compression didn't help, use original
                    (data, 0)
                }
            }
            CompressionAlgorithm::Zstd => {
                // TODO: Implement Zstd compression
                return Err(crate::large_data_transfer::LargeDataError::Compression(
                    "Zstd compression not yet implemented".to_string(),
                ));
            }
        };
        
        let id = ChunkId::from_data(&final_data);
        
        let info = ChunkInfo {
            id: id.clone(),
            original_size,
            compressed_size,
            compression,
            checksum,
            index,
        };
        
        Ok(Self {
            info,
            data: final_data,
        })
    }

    /// Get the chunk ID
    pub fn id(&self) -> &ChunkId {
        &self.info.id
    }

    /// Get compressed data size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Verify chunk integrity
    pub fn verify_integrity(&self) -> Result<(), ChunkError> {
        // Verify content hash
        let expected_id = ChunkId::from_data(&self.data);
        if expected_id != self.info.id {
            return Err(ChunkError::IntegrityCheckFailed(
                "Content hash mismatch".to_string(),
            ));
        }

        // Verify data is not corrupted by decompressing and checking size/checksum
        let decompressed = self.decompress()?;
        
        if decompressed.len() != self.info.original_size as usize {
            return Err(ChunkError::IntegrityCheckFailed(
                "Decompressed size mismatch".to_string(),
            ));
        }

        let actual_checksum = crc32fast::hash(&decompressed);
        if actual_checksum != self.info.checksum {
            return Err(ChunkError::IntegrityCheckFailed(
                "Checksum mismatch".to_string(),
            ));
        }

        Ok(())
    }

    /// Decompress chunk data
    pub fn decompress(&self) -> Result<Vec<u8>, ChunkError> {
        match self.info.compression {
            CompressionAlgorithm::None => Ok(self.data.clone()),
            CompressionAlgorithm::Lz4 => {
                if self.info.compressed_size == 0 {
                    // Not actually compressed
                    Ok(self.data.clone())
                } else {
                    lz4_flex::decompress_size_prepended(&self.data)
                        .map_err(|e| ChunkError::DecompressionFailed(e.to_string()))
                }
            }
            CompressionAlgorithm::Zstd => {
                Err(ChunkError::DecompressionFailed(
                    "Zstd decompression not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Get original (decompressed) data
    pub fn original_data(&self) -> Result<Vec<u8>, ChunkError> {
        self.decompress()
    }

    /// Get compression statistics
    pub fn compression_stats(&self) -> CompressionStats {
        CompressionStats {
            algorithm: self.info.compression,
            original_size: self.info.original_size,
            compressed_size: if self.info.is_compressed() {
                self.info.compressed_size
            } else {
                self.info.original_size
            },
            ratio: self.info.compression_ratio(),
            savings_bytes: if self.info.is_compressed() {
                self.info.original_size - self.info.compressed_size
            } else {
                0
            },
        }
    }
}

/// Compression statistics for a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub algorithm: CompressionAlgorithm,
    pub original_size: u32,
    pub compressed_size: u32,
    pub ratio: f32,
    pub savings_bytes: u32,
}

impl CompressionStats {
    /// Get compression savings as percentage
    pub fn savings_percentage(&self) -> f32 {
        if self.original_size == 0 {
            return 0.0;
        }
        (1.0 - self.ratio) * 100.0
    }
}

/// Chunk-related errors
#[derive(Error, Debug)]
pub enum ChunkError {
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    
    #[error("Integrity check failed: {0}")]
    IntegrityCheckFailed(String),
    
    #[error("Invalid chunk data: {0}")]
    InvalidData(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Utility functions for chunk operations
pub struct ChunkUtils;

impl ChunkUtils {
    /// Split data into chunks of specified size
    pub fn split_data(
        data: &[u8],
        chunk_size: u32,
        compression: CompressionAlgorithm,
    ) -> LargeDataResult<Vec<DataChunk>> {
        if chunk_size == 0 {
            return Err(crate::large_data_transfer::LargeDataError::Config(
                "Chunk size cannot be zero".to_string(),
            ));
        }

        let mut chunks = Vec::new();
        let chunk_size = chunk_size as usize;
        
        for (index, chunk_data) in data.chunks(chunk_size).enumerate() {
            let chunk = DataChunk::new(
                chunk_data.to_vec(),
                index as u32,
                compression,
            )?;
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    /// Reassemble chunks back into original data
    pub fn reassemble_chunks(chunks: &[DataChunk]) -> Result<Vec<u8>, ChunkError> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        // Sort chunks by index
        let mut sorted_chunks = chunks.to_vec();
        sorted_chunks.sort_by_key(|c| c.info.index);

        // Verify sequence is complete
        for (i, chunk) in sorted_chunks.iter().enumerate() {
            if chunk.info.index != i as u32 {
                return Err(ChunkError::InvalidData(
                    format!("Missing chunk at index {}", i),
                ));
            }
        }

        // Decompress and concatenate
        let mut result = Vec::new();
        for chunk in sorted_chunks {
            let decompressed = chunk.decompress()?;
            result.extend_from_slice(&decompressed);
        }

        Ok(result)
    }

    /// Calculate total size of chunks (original data)
    pub fn total_original_size(chunks: &[DataChunk]) -> u64 {
        chunks.iter().map(|c| c.info.original_size as u64).sum()
    }

    /// Calculate total compressed size of chunks
    pub fn total_compressed_size(chunks: &[DataChunk]) -> u64 {
        chunks.iter().map(|c| c.size() as u64).sum()
    }

    /// Calculate overall compression ratio
    pub fn overall_compression_ratio(chunks: &[DataChunk]) -> f32 {
        let original = Self::total_original_size(chunks);
        let compressed = Self::total_compressed_size(chunks);
        
        if original == 0 {
            return 1.0;
        }
        
        compressed as f32 / original as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_id_creation() {
        let data = b"hello world";
        let id1 = ChunkId::from_data(data);
        let id2 = ChunkId::from_data(data);
        assert_eq!(id1, id2);
        assert_eq!(id1.as_str().len(), 64); // SHA-256 hex length
    }

    #[test]
    fn test_chunk_id_from_hex() {
        let hex = "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3";
        let id = ChunkId::from_hex(hex).unwrap();
        assert_eq!(id.as_str(), hex);
        assert_eq!(id.short_hash(), "a665a459");
    }

    #[test]
    fn test_chunk_id_invalid_hex() {
        let result = ChunkId::from_hex("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_data_chunk_creation() {
        let data = b"hello world".to_vec();
        let chunk = DataChunk::new(data.clone(), 0, CompressionAlgorithm::None).unwrap();
        
        assert_eq!(chunk.info.original_size, data.len() as u32);
        assert_eq!(chunk.info.compressed_size, 0);
        assert_eq!(chunk.info.compression, CompressionAlgorithm::None);
        assert_eq!(chunk.info.index, 0);
    }

    #[test]
    fn test_data_chunk_compression() {
        let data = vec![b'A'; 1000]; // Highly compressible data
        let chunk = DataChunk::new(data.clone(), 0, CompressionAlgorithm::Lz4).unwrap();
        
        assert_eq!(chunk.info.original_size, 1000);
        assert!(chunk.info.compressed_size > 0);
        assert!(chunk.info.compressed_size < 1000);
        assert!(chunk.info.is_compressed());
        
        // Verify decompression works
        let decompressed = chunk.decompress().unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_chunk_integrity_verification() {
        let data = b"test data".to_vec();
        let chunk = DataChunk::new(data, 0, CompressionAlgorithm::None).unwrap();
        
        // Should pass integrity check
        assert!(chunk.verify_integrity().is_ok());
        
        // Corrupt the data and verify it fails
        let mut corrupted_chunk = chunk.clone();
        corrupted_chunk.data[0] = corrupted_chunk.data[0].wrapping_add(1);
        assert!(corrupted_chunk.verify_integrity().is_err());
    }

    #[test]
    fn test_chunk_utils_split_and_reassemble() {
        let original_data = b"This is a test string that will be split into chunks".to_vec();
        let chunk_size = 10;
        
        // Split into chunks
        let chunks = ChunkUtils::split_data(&original_data, chunk_size, CompressionAlgorithm::None).unwrap();
        assert!(!chunks.is_empty());
        
        // Reassemble
        let reassembled = ChunkUtils::reassemble_chunks(&chunks).unwrap();
        assert_eq!(reassembled, original_data);
    }

    #[test]
    fn test_compression_stats() {
        let data = vec![b'A'; 1000];
        let chunk = DataChunk::new(data, 0, CompressionAlgorithm::Lz4).unwrap();
        
        let stats = chunk.compression_stats();
        assert_eq!(stats.original_size, 1000);
        assert!(stats.compressed_size < 1000);
        assert!(stats.ratio < 1.0);
        assert!(stats.savings_percentage() > 0.0);
    }

    #[test]
    fn test_chunk_utils_size_calculations() {
        let data = vec![b'A'; 100];
        let chunks = ChunkUtils::split_data(&data, 30, CompressionAlgorithm::None).unwrap();
        
        assert_eq!(ChunkUtils::total_original_size(&chunks), 100);
        assert_eq!(ChunkUtils::total_compressed_size(&chunks), 100);
        assert_eq!(ChunkUtils::overall_compression_ratio(&chunks), 1.0);
    }
} 