//! Defines the error types for the P2P service.

use thiserror::Error;

/// Errors that can occur within the P2P service.
#[derive(Debug, Error)]
pub enum P2PError {
    #[error("Network connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Message serialization/deserialization failed: {0}")]
    SerializationFailed(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("P2P Service is not running or has been shut down")]
    ServiceNotStarted,

    #[error("Internal channel communication error: {0}")]
    ChannelError(String),

    #[error("Libp2p transport error: {0}")]
    TransportError(String),

    #[error("An I/O error occurred: {0}")]
    IoError(String),

    #[error("A generic network error occurred: {0}")]
    Network(String),
}

impl From<std::io::Error> for P2PError {
    fn from(e: std::io::Error) -> Self {
        P2PError::IoError(e.to_string())
    }
} 