use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
    #[error("Insufficient liquidity: required {required}, available {available}")]
    InsufficientLiquidity { required: u64, available: u64 },
    #[error("Invalid bridge transaction: {0}")]
    InvalidTransaction(String),
    #[error("Bridge validation failed: {0}")]
    ValidationFailed(String),
    #[error("Cross-chain timeout: {0}")]
    Timeout(String),
    #[error("Bridge security error: {0}")]
    SecurityError(String),
}

pub type BridgeResult<T> = Result<T, BridgeError>; 