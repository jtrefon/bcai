//! Defines error types related to chunking.

use thiserror::Error;

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