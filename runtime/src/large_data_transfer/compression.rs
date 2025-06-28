//! Compression Implementation for Large Data Transfer
//!
//! Provides helper functions for compressing and decompressing data.  The
//! implementation is intentionally simple but fully functional using the LZ4
//! algorithm.  Additional algorithms can be added in later phases.

use crate::large_data_transfer::{
    config::CompressionAlgorithm,
    error::{LargeDataError, LargeDataResult},
};
use std::io::Read;

/// Compression helper utilities.
pub struct CompressionUtils;

impl CompressionUtils {
    /// Compress `data` according to the selected algorithm.
    pub fn compress(data: &[u8], algo: CompressionAlgorithm) -> LargeDataResult<Vec<u8>> {
        match algo {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Lz4 => Ok(lz4_flex::compress_prepend_size(data)),
            CompressionAlgorithm::Zstd => zstd::bulk::compress(data, 3)
                .map_err(|e| LargeDataError::Compression(e.to_string())),
        }
    }

    /// Decompress `data` using the specified algorithm.
    pub fn decompress(data: &[u8], algo: CompressionAlgorithm) -> LargeDataResult<Vec<u8>> {
        match algo {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Lz4 => lz4_flex::decompress_size_prepended(data)
                .map_err(|e| LargeDataError::Compression(e.to_string())),
            CompressionAlgorithm::Zstd => {
                let mut decoder = zstd::Decoder::new(data)
                    .map_err(|e| LargeDataError::Compression(e.to_string()))?;
                let mut buf = Vec::new();
                decoder
                    .read_to_end(&mut buf)
                    .map_err(|e| LargeDataError::Compression(e.to_string()))?;
                Ok(buf)
            }
        }
    }
}
