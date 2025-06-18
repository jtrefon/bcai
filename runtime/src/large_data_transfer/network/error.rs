//! Error types for the network integration layer.

use crate::large_data_transfer::LargeDataError;
use thiserror::Error;

/// Network integration errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Network unreachable")]
    NetworkUnreachable,
    #[error("Transfer timeout")]
    TransferTimeout,
    #[error("Bandwidth limit exceeded")]
    BandwidthLimitExceeded,
    #[error("Chunk routing failed: {0}")]
    ChunkRoutingFailed(String),
    #[error("Large data error: {0}")]
    LargeData(#[from] LargeDataError),
}

impl From<NetworkError> for LargeDataError {
    fn from(err: NetworkError) -> Self {
        LargeDataError::Network(err.to_string())
    }
} 