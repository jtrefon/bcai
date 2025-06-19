use crate::large_data_transfer::{chunk::{error::ChunkError, stats::CompressionStats}, config::CompressionAlgorithm};
use super::core::DataChunk;

impl DataChunk {
    /// Verify content hash, size, checksum, and compression metadata.
    pub fn verify_integrity(&self) -> Result<(), ChunkError> {
        let expected_id = crate::large_data_transfer::chunk::id::ChunkId::from_data(&self.data);
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

    /// Return compression statistics for analytics / monitoring.
    pub fn compression_stats(&self) -> CompressionStats {
        CompressionStats {
            algorithm: self.info.compression,
            original_size: self.info.original_size,
            compressed_size: if self.info.is_compressed() { self.info.compressed_size } else { self.info.original_size },
            ratio: self.info.compression_ratio(),
            savings_bytes: if self.info.is_compressed() { self.info.original_size - self.info.compressed_size } else { 0 },
        }
    }
} 