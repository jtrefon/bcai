//! Defines the primary error types for node operations.

use crate::{job_manager::JobManagerError, token::LedgerError};
use thiserror::Error;

/// Errors that can occur during `UnifiedNode` operations.
#[derive(Debug, Error)]
pub enum NodeError {
    #[error("Job management error: {0}")]
    JobManager(#[from] JobManagerError),

    #[error("Token ledger error: {0}")]
    Ledger(#[from] LedgerError),

    #[error("Networking error: {0}")]
    Network(#[from] crate::network::NetworkError),

    #[error("Insufficient stake for the requested operation")]
    InsufficientStake,

    #[error("Node does not meet the capability requirements for the job")]
    CapabilityMismatch,

    #[error("The training result could not be verified")]
    TrainingVerificationFailed,

    #[error("A network communication error occurred: {0}")]
    NetworkCommunication(String),

    #[error("Job not found: {0}")]
    JobNotFound(u64),

    #[error("The operation is not valid for the job's current state")]
    InvalidStateTransition,
} 