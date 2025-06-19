use crate::large_data_transfer::{chunk::{id::ChunkId, info::ChunkInfo, error::ChunkError}, config::CompressionAlgorithm, error::LargeDataError, LargeDataResult};
use super::core::DataChunk;

impl DataChunk {
    /// Build a new chunk from raw `data`, applying optional compression.
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
                    (data, 0, CompressionAlgorithm::None)
                }
            }
            CompressionAlgorithm::Zstd => {
                return Err(LargeDataError::Compression("Zstd not implemented".into()));
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

        Ok(DataChunk { id, data: final_data, info })
    }

    /// Decompress stored data back to original bytes (no-op if uncompressed).
    pub fn decompress(&self) -> Result<Vec<u8>, ChunkError> {
        match self.info.compression {
            CompressionAlgorithm::None => Ok(self.data.clone()),
            CompressionAlgorithm::Lz4 => lz4_flex::decompress_size_prepended(&self.data)
                .map_err(|e| ChunkError::DecompressionFailed(e.to_string())),
            CompressionAlgorithm::Zstd => Err(ChunkError::DecompressionFailed("Zstd not implemented".into())),
        }
    }
} 