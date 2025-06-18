//! Defines error types for the protocol layer.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("Invalid message received: {0}")]
    InvalidMessage(String),

    #[error("A peer violated the transfer protocol: {0}")]
    ProtocolViolation(String),

    #[error("Transfer session not found for content hash: {0}")]
    TransferNotFound(String),

    #[error("Invalid state transition or operation for current state: {0}")]
    StateError(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),
} 