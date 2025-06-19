//! Error types for the large data transfer module.

use crate::large_data_transfer::chunk;
use thiserror::Error;
use crate::large_data_transfer::protocol::error::TransferError;

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

impl From<TransferError> for LargeDataError {
    fn from(e: TransferError) -> Self {
        LargeDataError::Network(e.to_string())
    }
}

/// A specialized `Result` type for large data transfer operations.
pub type LargeDataResult<T> = Result<T, LargeDataError>; 