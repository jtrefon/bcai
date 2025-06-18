//! Defines the `DataChunk` type used during large data transfers.

use super::{
    error::ChunkError,
    id::ChunkId,
    info::ChunkInfo,
    stats::CompressionStats,
};
use crate::large_data_transfer::{config::CompressionAlgorithm, error::LargeDataError, LargeDataResult};
use serde::{Deserialize, Serialize};

/// A chunk of raw bytes identified by a unique `ChunkId`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChunk {
    pub id: ChunkId,
    pub data: Vec<u8>,
}

impl DataChunk {
    pub fn new(id: ChunkId, data: Vec<u8>) -> Self {
        Self { id, data }
    }

    /// Returns the size of the chunk in bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Create a new data chunk from a raw data slice.
    pub fn new_from_slice(
        data: Vec<u8>,
        index: u32,
        compression: CompressionAlgorithm,
    ) -> LargeDataResult<Self> {
        let original_size = data.len() as u32;
        let checksum = crc32fast::hash(&data);

        let (final_data, compressed_size, actual_compression) = match compression {
            CompressionAlgorithm::None => (data, 0, CompressionAlgorithm::None),
            CompressionAlgorithm::Lz4 => {
                let compressed = lz4_flex::compress_prepend_size(&data);
                if compressed.len() < data.len() {
                    (compressed, compressed.len() as u32, CompressionAlgorithm::Lz4)
                } else {
                    // Compression increased size, so store uncompressed.
                    (data, 0, CompressionAlgorithm::None)
                }
            }
            CompressionAlgorithm::Zstd => {
                // zstd is not yet implemented in this example.
                return Err(LargeDataError::Compression(
                    "Zstd compression not yet implemented".to_string(),
                ));
            }
        };

        let id = ChunkId::from_data(&final_data);

        let info = ChunkInfo {
            id: id.clone(),
            original_size,
            compressed_size,
            compression: actual_compression,
            checksum,
            index,
        };

        Ok(Self { id, data: final_data })
    }

    /// Verify the integrity of the chunk by checking its hash and checksum.
    pub fn verify_integrity(&self) -> Result<(), ChunkError> {
        let expected_id = ChunkId::from_data(&self.data);
        if expected_id != self.id {
            return Err(ChunkError::IntegrityCheckFailed("Content hash mismatch".into()));
        }

        let decompressed = self.decompress()?;
        if decompressed.len() != self.info.original_size as usize {
            return Err(ChunkError::IntegrityCheckFailed("Decompressed size mismatch".into()));
        }

        let actual_checksum = crc32fast::hash(&decompressed);
        if actual_checksum != self.info.checksum {
            return Err(ChunkError::IntegrityCheckFailed("Checksum mismatch".into()));
        }

        Ok(())
    }

    /// Decompress the chunk data if it is compressed.
    pub fn decompress(&self) -> Result<Vec<u8>, ChunkError> {
        match self.info.compression {
            CompressionAlgorithm::None => Ok(self.data.clone()),
            CompressionAlgorithm::Lz4 => {
                lz4_flex::decompress_size_prepended(&self.data)
                    .map_err(|e| ChunkError::DecompressionFailed(e.to_string()))
            }
            CompressionAlgorithm::Zstd => Err(ChunkError::DecompressionFailed(
                "Zstd decompression not yet implemented".to_string(),
            )),
        }
    }
    
    /// Get compression statistics for this chunk.
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