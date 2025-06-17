use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Block validation failed: {0}")]
    BlockValidationError(String),
    #[error("Invalid transaction signature")]
    InvalidSignature,
    #[error("Invalid transaction nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },
    #[error("Insufficient funds for sender. Required: {required}, Available: {available}")]
    InsufficientFunds { required: u64, available: u64 },
} 