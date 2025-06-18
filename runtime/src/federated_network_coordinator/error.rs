use crate::federated::FederatedError;
use thiserror::Error;

/// Federated network coordination errors
#[derive(Debug, Error)]
pub enum FederatedNetworkError {
    #[error("Federated learning error: {0}")]
    Federated(#[from] FederatedError),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Large data transfer error: {0}")]
    LargeDataTransfer(String),
    #[error("Insufficient participants for job {job_id}: need {required}, have {available}")]
    InsufficientParticipants { job_id: u64, required: usize, available: usize },
    #[error("Job not found: {0}")]
    JobNotFound(u64),
    #[error("Invalid training round: expected {expected}, got {actual}")]
    InvalidRound { expected: u32, actual: u32 },
} 